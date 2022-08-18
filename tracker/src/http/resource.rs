use std::{fmt::Display, path::Path};

/// Newtype wrapper for the paths of HTTP related files
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource<'a>(&'a str);

impl<'a> Resource<'a> {
    /// Create a new resource
    pub fn new(path: &'a str) -> Self {
        Resource(path)
    }
}

impl<'a> AsRef<Path> for Resource<'a> {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl<'a> Display for Resource<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
