use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};

use crate::template;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Forbidden(String),
    BadRequest(String),
    Internal(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal Error: {}", msg),
        }
    }
}

impl AppError {
    fn status_and_message(&self) -> (StatusCode, &str, &str) {
        match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "Not Found", msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "Forbidden", msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "Bad Request", msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error", msg),
        }
    }

    /// Return HTML error page for browser requests, JSON for AJAX requests.
    pub fn into_response_for(self, headers: &HeaderMap) -> Response {
        let is_xhr = headers
            .get("X-Requested-With")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| v == "XMLHttpRequest");

        let (status, title, message) = self.status_and_message();

        if is_xhr {
            let body = serde_json::json!({ "error": message });
            (status, axum::Json(body)).into_response()
        } else {
            let html = template::error_html(status.as_u16(), title, message);
            (status, Html(html)).into_response()
        }
    }
}

/// Default IntoResponse: returns JSON (used as fallback when headers aren't available).
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, _title, message) = self.status_and_message();
        let body = serde_json::json!({ "error": message });
        (status, axum::Json(body)).into_response()
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => AppError::NotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => AppError::Forbidden(err.to_string()),
            _ => AppError::Internal(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn not_found_status() {
        let resp = AppError::NotFound("gone".into()).into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn forbidden_status() {
        let resp = AppError::Forbidden("nope".into()).into_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn bad_request_status() {
        let resp = AppError::BadRequest("bad".into()).into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn internal_status() {
        let resp = AppError::Internal("oops".into()).into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn display_not_found() {
        let e = AppError::NotFound("test".into());
        assert_eq!(format!("{}", e), "Not Found: test");
    }

    #[test]
    fn display_forbidden() {
        let e = AppError::Forbidden("test".into());
        assert_eq!(format!("{}", e), "Forbidden: test");
    }

    #[test]
    fn display_bad_request() {
        let e = AppError::BadRequest("test".into());
        assert_eq!(format!("{}", e), "Bad Request: test");
    }

    #[test]
    fn from_io_not_found() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let app_err = AppError::from(io_err);
        assert!(matches!(app_err, AppError::NotFound(_)));
    }

    #[test]
    fn from_io_permission_denied() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let app_err = AppError::from(io_err);
        assert!(matches!(app_err, AppError::Forbidden(_)));
    }

    #[test]
    fn from_io_other() {
        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "broken");
        let app_err = AppError::from(io_err);
        assert!(matches!(app_err, AppError::Internal(_)));
    }
}
