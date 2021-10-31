use pulldown_cmark::{html, Event, Options, Parser, Tag};

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
    let mut title = String::new();
    let mut events = Vec::new();
    let mut in_heading_1 = false;

    Parser::new_ext(markdown, *PARSER_OPTIONS).for_each(|event| match event {
        Event::Start(Tag::Heading(level)) => {
            if level == 1 {
                in_heading_1 = true;
            } else {
                events.push(event);
            }
        }
        Event::End(Tag::Heading(level)) => {
            if level == 1 {
                in_heading_1 = false;
            } else {
                events.push(event);
            }
        }
        Event::Text(text) => {
            if in_heading_1 {
                title = text.to_string();
            } else {
                events.push(Event::Text(text));
            }
        }
        event => {
            events.push(event);
        }
    });

    let mut html = String::new();
    html::push_html(&mut html, events.into_iter());

    Conversion { html, title }
}
