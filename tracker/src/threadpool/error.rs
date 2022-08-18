use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ThreadPoolError {
    PoolCreationError,
}

impl Error for ThreadPoolError {}

impl Display for ThreadPoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreadPoolError::PoolCreationError => write!(f, "Invalid number of threads"),
        }
    }
}
