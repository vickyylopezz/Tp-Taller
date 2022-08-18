/// Newtype wrapper for the content of a HTTP resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Content(Vec<u8>);

impl Content {
    pub fn new(c: Vec<u8>) -> Self {
        Self(c)
    }

    /// Returns the length of the content
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<Content> for Vec<u8> {
    fn from(c: Content) -> Self {
        c.0
    }
}
