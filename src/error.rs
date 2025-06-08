use std::fmt::Display;
use thiserror::Error as ThisError;

use crate::pattern::Pattern;

#[derive(Debug)]
pub struct HttpError {
    pub status: u16,
    pub message: String,
}

impl HttpError {
    pub fn new(message: impl AsRef<str>, status: u16) -> Self {
        HttpError { message: message.as_ref().to_string(), status }
    }

    pub fn err<E: std::error::Error>(err: E) -> Self {
        HttpError::new(err.to_string(), 500)
    }

    pub fn status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    pub fn message(mut self, message: String) -> Self {
        self.message = message;
        self
    }
}

impl Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

// impl std::error::Error for HttpError {}

impl From<&str> for HttpError {
    fn from(value: &str) -> Self {
        Self::new(value, 500)
    }
}

impl From<String> for HttpError {
    fn from(value: String) -> Self {
        Self::new(value, 500)
    }
}

impl From<std::io::Error> for HttpError {
    fn from(value: std::io::Error) -> Self {
        Self::new(value.to_string(), 500)
    }
}

#[derive(ThisError, Debug)]
pub enum RegisterError {
    #[error("invalid pattern: {0}")]
    InvalidPattern(#[from] PatternError),
    #[error("pattern {0} already registered ")]
    DuplicatePattern(Pattern),
    #[error("conflicting dynamic segment: expected {{{0}}} but got {{{1}}}")]
    DuplicateDynamicSegment(String, String),
    #[error("conflicting wildcard segment")]
    DuplicateWildcardSegment,
}

#[derive(ThisError, Debug)]
pub enum PatternError {
    #[error("unsupported or invalid method")]
    UnsupportedMethod,
    #[error("missing path")]
    MissingPath,
    #[error("path must start with '/'")]
    PathMustStartWithSlash,
    #[error("invalid host {0}")]
    InvalidHost(String),
    #[error("found invalid chars")]
    InvalidChars,
    #[error("wildcard (*) is only allowed as last chunk")]
    WildcardPosition,
    #[error("invalid dynamic pattern definition")]
    InvalidDynamic,
}