use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};

use crate::template;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Forbidden(String),
    BadRequest(String),
    Conflict(String),
    Internal(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
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
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "Conflict", msg),
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
    fn error_status_codes() {
        let cases: Vec<(AppError, StatusCode)> = vec![
            (AppError::NotFound("gone".into()),    StatusCode::NOT_FOUND),
            (AppError::Forbidden("nope".into()),   StatusCode::FORBIDDEN),
            (AppError::BadRequest("bad".into()),   StatusCode::BAD_REQUEST),
            (AppError::Internal("oops".into()),    StatusCode::INTERNAL_SERVER_ERROR),
        ];
        for (err, expected) in cases {
            assert_eq!(err.into_response().status(), expected);
        }
    }

    #[test]
    fn display_format() {
        let cases = [
            (AppError::NotFound("test".into()),   "Not Found: test"),
            (AppError::Forbidden("test".into()),  "Forbidden: test"),
            (AppError::BadRequest("test".into()), "Bad Request: test"),
        ];
        for (err, expected) in cases {
            assert_eq!(format!("{}", err), expected);
        }
    }

    #[test]
    fn from_io_error() {
        use std::io::{Error, ErrorKind};
        let cases: Vec<(ErrorKind, fn(&AppError) -> bool)> = vec![
            (ErrorKind::NotFound,         |e| matches!(e, AppError::NotFound(_))),
            (ErrorKind::PermissionDenied, |e| matches!(e, AppError::Forbidden(_))),
            (ErrorKind::BrokenPipe,       |e| matches!(e, AppError::Internal(_))),
        ];
        for (kind, check) in cases {
            let app_err = AppError::from(Error::new(kind, "msg"));
            assert!(check(&app_err), "wrong variant for {:?}", kind);
        }
    }
}
