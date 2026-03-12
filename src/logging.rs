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
    let start = std::time::Instant::now();

    let response = next.run(request).await;

    let status = response.status().as_u16();
    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");

    let line = format!(
        "[{}] {} {} {} {} {:.1}ms",
        now, addr.ip(), method, uri, status, elapsed_ms,
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
