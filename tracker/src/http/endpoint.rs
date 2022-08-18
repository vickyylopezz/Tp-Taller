use std::fmt::Display;

/// Newtype wrapper for an endpoint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Endpoint<'a>(&'a str);

impl<'a> Endpoint<'a> {
    /// Creates a new endpoint from a string. As new endpoints
    /// shouldn't be added or change at runtime, if the endpoint it's
    /// invalid the program will panic.
    pub fn new(endpoint: &'a str) -> Self {
        assert!(endpoint.starts_with('/'));
        Self(endpoint)
    }
}

impl Display for Endpoint<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
