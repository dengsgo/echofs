use axum::body::Body;
use axum::http::{header, HeaderMap, Response, StatusCode};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

use crate::throttle::ThrottledRead;

fn stream_body<R: AsyncRead + Send + Unpin + 'static>(reader: R, speed_limit: Option<u64>) -> Body {
    match speed_limit {
        Some(limit) => {
            let throttled = ThrottledRead::new(reader, limit);
            Body::from_stream(ReaderStream::new(throttled))
        }
        None => Body::from_stream(ReaderStream::new(reader)),
    }
}

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
    speed_limit: Option<u64>,
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
            let body = stream_body(limited, speed_limit);

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
        let body = stream_body(file, speed_limit);

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

    /// Helper: assert parse_range returns Some with expected start/end.
    fn assert_range(header: &str, file_size: u64, expected_start: u64, expected_end: u64) {
        let r = parse_range(header, file_size)
            .unwrap_or_else(|| panic!("expected Some for {:?} (size={})", header, file_size));
        assert_eq!((r.start, r.end), (expected_start, expected_end),
            "range mismatch for {:?} (size={})", header, file_size);
    }

    #[test]
    fn valid_ranges() {
        // (header, file_size, expected_start, expected_end)
        let cases = [
            ("bytes=0-499",   1000,  0, 499),   // full range
            ("bytes=500-",    1000, 500, 999),   // open end
            ("bytes=-200",    1000, 800, 999),   // suffix range
            ("bytes=0-0",     1000,   0,   0),   // single byte
            ("bytes=0-9999",   500,   0, 499),   // end clamped to file size
            ("bytes=999-999", 1000, 999, 999),   // last byte
        ];
        for (header, size, start, end) in cases {
            assert_range(header, size, start, end);
        }
    }

    #[test]
    fn rejected_ranges() {
        // (header, file_size)
        let cases = [
            ("0-499",          1000), // no bytes= prefix
            ("bytes=1000-",    1000), // start >= file_size
            ("bytes=500-100",  1000), // start > end
            ("bytes=-0",       1000), // suffix zero
            ("bytes=-2000",    1000), // suffix > file_size
            ("bytes=0-0",         0), // empty file
            ("bytes=abc-def",  1000), // garbage values
            ("garbage",        1000), // not a range at all
        ];
        for (header, size) in cases {
            assert!(parse_range(header, size).is_none(),
                "expected None for {:?} (size={})", header, size);
        }
    }
}
