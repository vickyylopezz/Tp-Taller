use std::fmt::Display;

use super::content::Content;
/// Represents the possible HTTP status codes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatusCode {
    /// OK
    Code200,
    /// NOT FOUND
    Code404,
}

/// Response of the HTTP server
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Response(Vec<u8>);

impl Response {
    /// Creates a new response with the specified status code.
    pub fn new(s: StatusCode) -> Self {
        Self(format!("HTTP/1.1 {}", s).into_bytes())
    }

    /// Adds the content length field
    pub fn content_length(mut self, length: usize) -> Self {
        let mut content = format!("\r\nContent-Length: {}\r\n\r\n", length).into_bytes();
        self.0.append(&mut content);
        self
    }
    /// Adds the content type field
    pub fn content_type(mut self, content_type: &str) -> Self {
        let mut content = format!("\r\nContent-Type: {}\r\n\r\n", content_type).into_bytes();
        self.0.append(&mut content);
        self
    }
    /// Adds the body of the response
    pub fn content(mut self, content: Content) -> Self {
        self.0.append(&mut content.into());
        self
    }
    /// Builds the response
    pub fn response(self) -> Vec<u8> {
        self.0
    }
}

impl Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusCode::Code200 => write!(f, "200 OK"),
            StatusCode::Code404 => write!(f, "404 NOT FOUND"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn build_response_with_status_code() {
        let r1 = Response::new(StatusCode::Code200);
        let r2 = Response::new(StatusCode::Code404);

        assert_eq!(r1.response(), b"HTTP/1.1 200 OK");
        assert_eq!(r2.response(), b"HTTP/1.1 404 NOT FOUND");
    }

    #[test]
    fn build_response_with_content() {
        let content = Content::new(b"abc".to_vec());
        let r = Response::new(StatusCode::Code200)
            .content_length(content.len())
            .content(Content::new(b"abc".to_vec()));
        assert_eq!(
            r.response(),
            b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nabc"
        )
    }
}
