use std::{collections::HashMap, fs};

use super::{content::Content, endpoint::Endpoint, error::ApiError, resource::Resource};

/// Constructs the API, to be used with the HTTP server. The API must
/// be defined at compile time and mustn't be changed at runtime. A
/// resource for the error case must be provided.
#[derive(Debug, Clone)]
pub struct APIBuilder<'a> {
    inner: HashMap<Endpoint<'a>, Resource<'a>>,
    error_resource: Resource<'a>,
}

impl<'a> Default for APIBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> APIBuilder<'a> {
    /// Initializes the API
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            error_resource: Resource::new(""),
        }
    }

    /// Adds a new endpoint to the API, associated with the specified
    /// resource.
    pub fn add(&mut self, endpoint: Endpoint<'a>, resource: Resource<'a>) {
        self.inner.insert(endpoint, resource);
    }

    /// Defines the resource for the error cases.
    pub fn not_found(&mut self, resource: Resource<'a>) {
        self.error_resource = resource;
    }

    /// Gets the content associated with the specific endpoint, if it
    /// exist. Otherwise returns an error.
    pub fn get(&self, endpoint: Endpoint<'a>) -> Result<Content, ApiError> {
        let resource = self.inner.get(&endpoint);
        match resource {
            Some(s) => fs::read(s).map_or(
                Err(ApiError::InvalidResourceRead(endpoint.to_string())),
                |s| Ok(Content::new(s)),
            ),
            None => fs::read(&self.error_resource).map_or(
                Err(ApiError::InvalidResourceRead(
                    self.error_resource.to_string(),
                )),
                |s| {
                    Err(ApiError::ResourceNotFound(
                        Content::new(s),
                        endpoint.to_string(),
                    ))
                },
            ),
        }
    }
}
