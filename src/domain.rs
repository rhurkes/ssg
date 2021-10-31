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
    pub title: String,
}
