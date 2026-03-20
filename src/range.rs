use axum::body::Body;
use axum::http::{header, HeaderMap, Response, StatusCode};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

pub struct RangeSpec {
    pub start: u64,
    pub end: u64,
}

pub fn parse_range(range_header: &str, file_size: u64) -> Option<RangeSpec> {
    let range_str = range_header.strip_prefix("bytes=")?;
    let parts: Vec<&str> = range_str.splitn(2, '-').collect();
    if parts.len() != 2 {
        return None;
    }

    let start_str = parts[0].trim();
    let end_str = parts[1].trim();

    if start_str.is_empty() {
        // suffix range: -500 means last 500 bytes
        let suffix_len: u64 = end_str.parse().ok()?;
        if suffix_len == 0 || suffix_len > file_size {
            return None;
        }
        Some(RangeSpec {
            start: file_size - suffix_len,
            end: file_size - 1,
        })
    } else {
        let start: u64 = start_str.parse().ok()?;
        let end = if end_str.is_empty() {
            file_size - 1
        } else {
            end_str.parse().ok()?
        };

        if start > end || start >= file_size {
            return None;
        }

        let end = end.min(file_size - 1);
        Some(RangeSpec { start, end })
    }
}

pub async fn build_range_response(
    path: &Path,
    headers: &HeaderMap,
    content_type: &str,
) -> Result<Response<Body>, std::io::Error> {
    let metadata = tokio::fs::metadata(path).await?;
    let file_size = metadata.len();

    let range_header = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok());

    if let Some(range_str) = range_header {
        if let Some(range) = parse_range(range_str, file_size) {
            let content_length = range.end - range.start + 1;

            let mut file = File::open(path).await?;
            file.seek(std::io::SeekFrom::Start(range.start)).await?;
            let limited = file.take(content_length);
            let stream = ReaderStream::new(limited);
            let body = Body::from_stream(stream);

            let response = Response::builder()
                .status(StatusCode::PARTIAL_CONTENT)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::CONTENT_LENGTH, content_length)
                .header(header::ACCEPT_RANGES, "bytes")
                .header(
                    header::CONTENT_RANGE,
                    format!("bytes {}-{}/{}", range.start, range.end, file_size),
                )
                .body(body)
                .expect("valid 206 response with known headers");

            Ok(response)
        } else {
            // Invalid range - 416
            let response = Response::builder()
                .status(StatusCode::RANGE_NOT_SATISFIABLE)
                .header(header::CONTENT_RANGE, format!("bytes */{}", file_size))
                .body(Body::empty())
                .expect("valid 416 response with known headers");
            Ok(response)
        }
    } else {
        // No range header - full file
        let file = File::open(path).await?;
        let stream = ReaderStream::new(file);
        let body = Body::from_stream(stream);

        let response = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CONTENT_LENGTH, file_size)
            .header(header::ACCEPT_RANGES, "bytes")
            .body(body)
            .expect("valid 200 response with known headers");

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_range() {
        let r = parse_range("bytes=0-499", 1000).unwrap();
        assert_eq!(r.start, 0);
        assert_eq!(r.end, 499);
    }

    #[test]
    fn parse_open_end_range() {
        let r = parse_range("bytes=500-", 1000).unwrap();
        assert_eq!(r.start, 500);
        assert_eq!(r.end, 999);
    }

    #[test]
    fn parse_suffix_range() {
        let r = parse_range("bytes=-200", 1000).unwrap();
        assert_eq!(r.start, 800);
        assert_eq!(r.end, 999);
    }

    #[test]
    fn parse_single_byte_range() {
        let r = parse_range("bytes=0-0", 1000).unwrap();
        assert_eq!(r.start, 0);
        assert_eq!(r.end, 0);
    }

    #[test]
    fn parse_end_clamped_to_file_size() {
        let r = parse_range("bytes=0-9999", 500).unwrap();
        assert_eq!(r.start, 0);
        assert_eq!(r.end, 499);
    }

    #[test]
    fn parse_last_byte() {
        let r = parse_range("bytes=999-999", 1000).unwrap();
        assert_eq!(r.start, 999);
        assert_eq!(r.end, 999);
    }

    #[test]
    fn reject_no_bytes_prefix() {
        assert!(parse_range("0-499", 1000).is_none());
    }

    #[test]
    fn reject_start_ge_file_size() {
        assert!(parse_range("bytes=1000-", 1000).is_none());
    }

    #[test]
    fn reject_start_gt_end() {
        assert!(parse_range("bytes=500-100", 1000).is_none());
    }

    #[test]
    fn reject_suffix_zero() {
        assert!(parse_range("bytes=-0", 1000).is_none());
    }

    #[test]
    fn reject_suffix_gt_file_size() {
        assert!(parse_range("bytes=-2000", 1000).is_none());
    }

    #[test]
    fn reject_empty_file() {
        assert!(parse_range("bytes=0-0", 0).is_none());
    }

    #[test]
    fn reject_garbage() {
        assert!(parse_range("bytes=abc-def", 1000).is_none());
        assert!(parse_range("garbage", 1000).is_none());
    }
}
