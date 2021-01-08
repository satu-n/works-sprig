use actix_web::{error::{BlockingError, ResponseError}, http::StatusCode, HttpResponse};
use combine::error::StringStreamError;
use derive_more::Display;
use diesel::result::{DatabaseErrorKind, Error as DbError};

#[derive(Debug, Display)]
pub enum ServiceError {
    #[display(fmt = "Oops, {}", _0)]
    BadRequest(String),

    Unauthorized,
    InternalServerError,
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::BadRequest(msg)     => HttpResponse::BadRequest().body(msg),
            ServiceError::Unauthorized        => HttpResponse::Unauthorized().finish(),
            ServiceError::InternalServerError => HttpResponse::InternalServerError().finish(),
        }
    }
    fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::BadRequest(_)       => StatusCode::BAD_REQUEST,
            ServiceError::Unauthorized        => StatusCode::UNAUTHORIZED,
            ServiceError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<BlockingError<Self>> for ServiceError {
    fn from(error: BlockingError<Self>) -> Self {
        dbg!(&error);
        match error {
            BlockingError::Error(service_error) => service_error,
            BlockingError::Canceled => Self::InternalServerError,
        }
    }
}

impl From<DbError> for ServiceError {
    fn from(error: DbError) -> Self {
        dbg!(&error);
        match error {
            DbError::DatabaseError(kind, info) => {
                if let DatabaseErrorKind::UniqueViolation = kind {
                    return Self::BadRequest(info.message().into());
                }
                Self::InternalServerError
            }
            _ => Self::InternalServerError,
        }
    }
}

impl From<regex::Error> for ServiceError {
    fn from(error: regex::Error) -> Self {
        dbg!(&error);
        match error {
            regex::Error::Syntax(s) => Self::BadRequest(format!(
                "regex error: {}",
                s,
            )),
            regex::Error::CompiledTooBig(_) => Self::BadRequest(format!(
                "regex compiled too big."
            )),
            _ => Self::InternalServerError,
        }
    }
}

impl From<StringStreamError> for ServiceError {
    fn from(error: StringStreamError) -> Self {
        dbg!(&error);
        match error {
            StringStreamError::UnexpectedParse => Self::BadRequest(format!(
                "there seems to be a syntax error.",
            )),
            _ => Self::InternalServerError,
        }
    }
}
