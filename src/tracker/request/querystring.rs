pub struct Querystring(pub String);

impl Querystring {
    pub fn get_querystring(&self) -> String {
        self.0.clone()
    }
}
