use axum::{http::StatusCode, response::IntoResponse};

pub enum MyError {
    BadRequest,
    TaskNotFound,
    InternalServerError,
}

impl IntoResponse for MyError {
    fn into_response(self) -> axum::response::Response {
        let (status, _) = match self {
            Self::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::BadRequest => (StatusCode::BAD_REQUEST, "Bad Request"),
            Self::TaskNotFound => (StatusCode::NOT_FOUND, "Task Not Found"),
        };
        (status, "error").into_response()
    }
}
