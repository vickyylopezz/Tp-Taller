use super::info::{Info, InfoMode};
use super::metainfo::Metainfo;

pub struct MetainfoBuilder {
    announce: String,
    announce_list: Option<Vec<String>>,
    comment: Option<String>,
    created_by: Option<String>,
    creation_date: Option<i64>,
    encoding: Option<String>,
    info: Info,
}

impl MetainfoBuilder {
    pub fn new() -> Self {
        Self {
            info: Info(InfoMode::Empty),
            announce: String::new(),
            announce_list: None,
            creation_date: None,
            comment: None,
            created_by: None,
            encoding: None,
        }
    }

    pub fn info(&'_ mut self, i: Info) -> &'_ mut Self {
        self.info = i;
        self
    }

    pub fn announce(&'_ mut self, s: String) -> &'_ mut Self {
        self.announce = s;
        self
    }

    pub fn announce_list(&'_ mut self, urls: Option<Vec<String>>) -> &'_ mut Self {
        self.announce_list = urls;
        self
    }

    pub fn creation_date(&'_ mut self, date: Option<i64>) -> &'_ mut Self {
        self.creation_date = date;
        self
    }

    pub fn comment(&'_ mut self, c: Option<String>) -> &'_ mut Self {
        self.comment = c;
        self
    }

    pub fn created_by(&'_ mut self, creator: Option<String>) -> &'_ mut Self {
        self.created_by = creator;
        self
    }

    pub fn encoding(&'_ mut self, e: Option<String>) -> &'_ mut Self {
        self.encoding = e;
        self
    }

    pub fn build(self) -> Metainfo {
        Metainfo {
            announce: self.announce,
            announce_list: self.announce_list,
            comment: self.comment,
            created_by: self.created_by,
            creation_date: self.creation_date,
            encoding: self.encoding,
            info: self.info,
        }
    }
}
