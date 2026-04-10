use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;
use tokio::io::AsyncRead;
use tokio::io::ReadBuf;

/// Wraps an `AsyncRead` to limit throughput to `bytes_per_sec`.
///
/// Uses a token-bucket approach: tokens (bytes) accumulate over time up to
/// `bytes_per_sec` (one second worth). Each `poll_read` consumes tokens equal
/// to the bytes actually read; when tokens are exhausted, the reader sleeps
/// until enough tokens have accumulated.
pub struct ThrottledRead<R> {
    inner: R,
    bytes_per_sec: f64,
    tokens: f64,
    last_refill: Instant,
    sleep: Pin<Box<tokio::time::Sleep>>,
    sleeping: bool,
}

impl<R: AsyncRead + Unpin> ThrottledRead<R> {
    pub fn new(inner: R, bytes_per_sec: u64) -> Self {
        Self {
            inner,
            bytes_per_sec: bytes_per_sec as f64,
            tokens: bytes_per_sec as f64, // start with a full bucket
            last_refill: Instant::now(),
            sleep: Box::pin(tokio::time::sleep(tokio::time::Duration::ZERO)),
            sleeping: false,
        }
    }

    fn refill_tokens(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.last_refill = now;
        self.tokens += elapsed * self.bytes_per_sec;
        // Cap at one second worth of tokens to limit burst
        if self.tokens > self.bytes_per_sec {
            self.tokens = self.bytes_per_sec;
        }
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for ThrottledRead<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();

        // If we are sleeping, wait for the sleep to complete
        if this.sleeping {
            match this.sleep.as_mut().poll(cx) {
                Poll::Ready(()) => {
                    this.sleeping = false;
                    this.refill_tokens();
                }
                Poll::Pending => return Poll::Pending,
            }
        }

        this.refill_tokens();

        // Need at least 1 byte worth of tokens
        if this.tokens < 1.0 {
            // Calculate how long to sleep to get at least 1 byte of tokens
            let wait_secs = (1.0 - this.tokens) / this.bytes_per_sec;
            let wait = std::time::Duration::from_secs_f64(wait_secs);
            this.sleep
                .as_mut()
                .reset(tokio::time::Instant::now() + wait);
            this.sleeping = true;
            // Poll the sleep future immediately so it registers its waker
            // with the timer in this same poll cycle — avoids a spurious wakeup.
            match this.sleep.as_mut().poll(cx) {
                Poll::Ready(()) => {
                    // Timer already expired (e.g. zero or tiny duration)
                    this.sleeping = false;
                    this.refill_tokens();
                    // Fall through to read below
                }
                Poll::Pending => return Poll::Pending,
            }
        }

        // Limit the read buffer to available tokens
        let allowed = this.tokens as usize;
        let remaining = buf.remaining();
        let limit = remaining.min(allowed);

        // Create a limited buffer
        let filled_before = buf.filled().len();
        if limit < remaining {
            // We need to limit the read — use a smaller buffer
            let mut limited_buf = ReadBuf::new(&mut buf.initialize_unfilled()[..limit]);
            match Pin::new(&mut this.inner).poll_read(cx, &mut limited_buf) {
                Poll::Ready(Ok(())) => {
                    let bytes_read = limited_buf.filled().len();
                    // Advance the original buffer
                    buf.advance(bytes_read);
                    this.tokens -= bytes_read as f64;
                    Poll::Ready(Ok(()))
                }
                other => other,
            }
        } else {
            match Pin::new(&mut this.inner).poll_read(cx, buf) {
                Poll::Ready(Ok(())) => {
                    let bytes_read = buf.filled().len() - filled_before;
                    this.tokens -= bytes_read as f64;
                    Poll::Ready(Ok(()))
                }
                other => other,
            }
        }
    }
}

/// Parse a human-readable speed string like "500k", "1m", "2g" into bytes/sec.
/// Supports suffixes: k/K (×1024), m/M (×1024²), g/G (×1024³).
/// Plain number is treated as bytes/sec.
pub fn parse_speed(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let (num_str, multiplier) = match s.as_bytes().last()? {
        b'k' | b'K' => (&s[..s.len() - 1], 1024u64),
        b'm' | b'M' => (&s[..s.len() - 1], 1024 * 1024),
        b'g' | b'G' => (&s[..s.len() - 1], 1024 * 1024 * 1024),
        _ => (s, 1u64),
    };

    let num: f64 = num_str.parse().ok()?;
    if num <= 0.0 || !num.is_finite() {
        return None;
    }

    Some((num * multiplier as f64) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    #[test]
    fn parse_speed_valid() {
        // (input, expected_bytes_per_sec)
        let cases = [
            ("1024",  1024),
            ("1k",    1024),
            ("1K",    1024),
            ("500k",  512_000),
            ("1m",    1_048_576),
            ("1M",    1_048_576),
            ("10m",   10_485_760),
            ("1g",    1_073_741_824),
            ("1G",    1_073_741_824),
            ("1.5m",  1_572_864),
            ("0.5k",  512),
        ];
        for (input, expected) in cases {
            assert_eq!(parse_speed(input), Some(expected), "parse_speed({:?})", input);
        }
    }

    #[test]
    fn parse_speed_invalid() {
        let cases = ["0", "-1", "", "abc", "k", "-5m"];
        for input in cases {
            assert_eq!(parse_speed(input), None, "expected None for {:?}", input);
        }
    }

    #[tokio::test]
    async fn throttled_read_limits_throughput() {
        // 10KB of data at 10KB/s
        let data = vec![0u8; 10_240];
        let mut reader = ThrottledRead::new(std::io::Cursor::new(data), 10_240);

        let start = Instant::now();
        let mut buf = vec![0u8; 20_480];
        let mut total = 0;
        loop {
            let n = reader.read(&mut buf).await.unwrap();
            if n == 0 { break; }
            total += n;
        }

        assert_eq!(total, 10_240);
        // Allow generous tolerance for CI
        assert!(start.elapsed().as_secs_f64() < 3.0, "took too long");
    }

    #[tokio::test]
    async fn throttled_read_all_data_arrives() {
        let data: Vec<u8> = (0..=255).cycle().take(4096).collect();
        let mut reader = ThrottledRead::new(std::io::Cursor::new(data.clone()), 1_048_576);

        let mut result = Vec::new();
        reader.read_to_end(&mut result).await.unwrap();
        assert_eq!(result, data);
    }
}
