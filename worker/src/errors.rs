use std::fmt::{Display, Formatter};
use async_mailer::SmtpMailerError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use log::error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConfigMissingEnv(&'static str),
    ConfigWrongFormat(&'static str),
    Io(std::io::Error),
    Request(reqwest::Error),
    EmailSendFailed(async_mailer::mail_send::Error),
    SMTPFailed(SmtpMailerError)
}

// region:    --- From

impl From<async_mailer::mail_send::Error> for Error {
    fn from(value: async_mailer::mail_send::Error) -> Self {
        Error::EmailSendFailed(value)
    }
}

impl From<SmtpMailerError> for Error {
    fn from(value: SmtpMailerError) -> Self {
        Error::SMTPFailed(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Request(err)
    }
}

// endregion: --- From

// region:    --- Error boilerplate
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
// endregion: --- Error boilerplate


impl IntoResponse for Error {
    fn into_response(self) -> Response {
        error!("{:?}", self);
        let body = self.to_string();
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}