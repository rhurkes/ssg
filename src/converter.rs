use pulldown_cmark::{html, Options, Parser};

lazy_static! {
    static ref PARSER_OPTIONS: Options = {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options
    };
}

pub struct Conversion {
    pub html: String,
    pub title: String,
}

pub fn convert(markdown: &str) -> Conversion {
    let title = String::new();
    let parser = Parser::new_ext(markdown, *PARSER_OPTIONS);
    let mut html = String::new();
    html::push_html(&mut html, parser);

    Conversion { html, title }
}
