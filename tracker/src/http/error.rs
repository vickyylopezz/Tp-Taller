use std::{error::Error, fmt::Display};

use super::content::Content;

/// Represents the errors that may occur while building the api
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    /// Recoverable error, the endpoint doesn't exist. Provides a
    /// default resource
    ResourceNotFound(Content, String),
    /// Unrecoverable error, the resource couldn't be accessed.
    InvalidResourceRead(String),
}

impl Error for ApiError {}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::ResourceNotFound(_, ctx) => writeln!(f, "Resource {} couldn't be found", ctx),
            ApiError::InvalidResourceRead(ctx) => {
                writeln!(f, "Resource associated with {} couldn't be accessed", ctx)
            }
        }
    }
}
