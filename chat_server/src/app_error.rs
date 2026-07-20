use axum::{
    body::Body,
    response::{IntoResponse, Response},
};

pub enum AppError {
    BadRequest(String),
    ServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let msg = self.message();
        let res = Response::builder();

        let res = match self {
            AppError::BadRequest(_) => res.status(400),
            AppError::ServerError(_) => res.status(500),
        };

        res.body(Body::new(msg.to_string()))
            .expect("The data we specified is correct")
    }
}

#[allow(unused)]
impl AppError {
    #[allow(clippy::needless_pass_by_value)]
    pub fn bad_request(msg: impl ToString) -> Self {
        Self::BadRequest(msg.to_string())
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn server_error(msg: impl ToString) -> Self {
        Self::ServerError(msg.to_string())
    }

    fn message(&self) -> &str {
        match self {
            AppError::BadRequest(msg) | AppError::ServerError(msg) => msg,
        }
    }
}
