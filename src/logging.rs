use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub enum LogTarget {
    Stdout,
    Off,
    File(Arc<Mutex<std::fs::File>>),
}

impl LogTarget {
    pub fn from_arg(value: &str) -> Self {
        match value {
            "stdout" => LogTarget::Stdout,
            "off" => LogTarget::Off,
            path => {
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .unwrap_or_else(|e| {
                        eprintln!("Failed to open log file '{}': {}", path, e);
                        std::process::exit(1);
                    });
                LogTarget::File(Arc::new(Mutex::new(file)))
            }
        }
    }
}

pub async fn access_log(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    log_target: axum::extract::State<LogTarget>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let is_xhr = request
        .headers()
        .get("X-Requested-With")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "XMLHttpRequest")
        .unwrap_or(false);
    let start = std::time::Instant::now();

    let response = next.run(request).await;

    let status = response.status().as_u16();
    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    let kind = if is_xhr { 'A' } else { 'P' };

    let line = format!(
        "[{}] {} {} {} {} {} {:.1}ms",
        now, addr.ip(), method, kind, uri, status, elapsed_ms,
    );

    match log_target.0.clone() {
        LogTarget::Stdout => {
            println!("{}", line);
        }
        LogTarget::Off => {}
        LogTarget::File(file) => {
            let mut f = file.lock().await;
            let _ = writeln!(f, "{}", line);
        }
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_arg_stdout() {
        let target = LogTarget::from_arg("stdout");
        assert!(matches!(target, LogTarget::Stdout));
    }

    #[test]
    fn from_arg_off() {
        let target = LogTarget::from_arg("off");
        assert!(matches!(target, LogTarget::Off));
    }

    #[test]
    fn from_arg_file() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        let target = LogTarget::from_arg(&path);
        assert!(matches!(target, LogTarget::File(_)));
    }
}
