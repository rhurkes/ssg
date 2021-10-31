use std::collections::HashMap;
use time::OffsetDateTime;

pub struct File {
    pub created: OffsetDateTime,
    pub filename: String,
    pub markdown: String,
}

pub struct Content {
    pub created: String,
    pub filename: String,
    pub html: String,
    pub tags: Vec<String>,
    pub title: String,
}

pub struct PostMaps {
    pub posts: HashMap<String, Content>,
    pub tag_posts: HashMap<String, Vec<String>>,
}
