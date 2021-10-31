use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Options, Parser, Tag};
use regex::Regex;
use std::collections::HashMap;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

lazy_static! {
    static ref PARSER_OPTIONS: Options = {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options
    };
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref SYNTAX_THEME: Theme =
        ThemeSet::get_theme("resources/Tomorrow.tmTheme").expect("Unable to load theme");
    static ref WORD_REGEX: Regex = Regex::new(r"(\w+)").unwrap();
}

pub struct Conversion {
    pub html: String,
    pub image_count: usize,
    pub title: String,
    pub word_count: usize,
}

pub fn convert(markdown: &str) -> Conversion {
    let mut title = String::new();
    let mut events = Vec::new();
    let mut in_heading_1 = false;
    // Keep state of whether inside code block, and capture language
    let mut in_code_block: Option<String> = None;
    let mut code = String::new();
    // Keep state of whether inside footnote, and capture reference
    let mut in_footnote: Option<String> = None;
    let mut footnotes: HashMap<String, String> = HashMap::new();
    let mut sidenote = String::new();
    let mut image_count = 0;
    let word_count = WORD_REGEX.find_iter(markdown).count();

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
        Event::Start(Tag::CodeBlock(block)) => {
            in_code_block = match block {
                CodeBlockKind::Fenced(tag) => Some(tag.into_string()),
                _ => Some(String::new()),
            }
        }
        Event::End(Tag::CodeBlock(_)) => {
            if let Some(language) = &in_code_block {
                let syntax = SYNTAX_SET
                    .find_syntax_by_token(language)
                    .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
                let html = highlighted_html_for_string(&code, &SYNTAX_SET, syntax, &SYNTAX_THEME);
                let html = CowStr::Boxed(html.into_boxed_str());
                events.push(Event::Html(html));
            }

            code = String::new();
            in_code_block = None;
        }
        Event::Code(code) => {
            if in_footnote.is_some() {
                let html = format!("<code>{}</code>", code);
                sidenote.push_str(&html);
            } else {
                events.push(Event::Code(code));
            }
        }
        Event::Text(text) => {
            if in_code_block.is_some() {
                code.push_str(&text);
            } else if in_heading_1 {
                title = text.to_string();
            } else if in_footnote.is_some() {
                footnotes.insert(in_footnote.as_ref().unwrap().to_string(), text.to_string());
                sidenote.push_str(&text);
            } else {
                events.push(Event::Text(text));
            }
        }
        Event::FootnoteReference(text) => {
            let token = format!("{{{{{} footnote}}}}", text);
            let html = CowStr::Boxed(token.into_boxed_str());
            events.push(Event::Html(html));
        }
        Event::Start(Tag::FootnoteDefinition(text)) => {
            in_footnote = Some(text.to_string());
        }
        Event::End(Tag::FootnoteDefinition(_)) => {
            footnotes.insert(
                in_footnote.as_ref().unwrap().to_string(),
                sidenote.to_string(),
            );
            in_footnote = None;
            sidenote = String::new();
        }
        Event::Start(Tag::Image(link_type, url, title)) => {
            image_count += 1;
            if in_footnote.is_some() {
                let image_html = format!("<img src=\"{}\" />", url);
                sidenote.push_str(&image_html);
            } else {
                events.push(Event::Start(Tag::Image(link_type, url, title)));
            }
        }
        Event::End(Tag::Image(_, _, _)) => {}
        Event::Start(Tag::Link(_, url, _)) => {
            let link = format!("<a href=\"{}\" target=\"_blank\">", url);

            if in_footnote.is_some() {
                sidenote.push_str(&link);
            } else {
                let html = CowStr::Boxed(link.into_boxed_str());
                events.push(Event::Html(html));
            }
        }
        Event::End(Tag::Link(_, _, _)) => {
            if in_footnote.is_some() {
                sidenote.push_str("</a>");
            } else {
                let link = "</a>".to_string();
                let html = CowStr::Boxed(link.into_boxed_str());
                events.push(Event::Html(html));
            }
        }
        event => {
            events.push(event);
        }
    });

    let mut html = String::new();
    html::push_html(&mut html, events.into_iter());

    // Parse entirely, then insert footnotes as sidenotes in the correct place in the document
    footnotes.iter().for_each(|(reference, content)| {
        let token = format!("{{{{{} footnote}}}}", reference);
        let sup = if reference.parse::<f64>().is_ok() {
            format!("<sup>{}</sup>", reference)
        } else {
            String::new()
        };

        let sidenote = format!("{}<span class=\"sidenote\">{}{}</span>", sup, sup, content);
        html = html.replace(&token, &sidenote);
    });

    Conversion {
        html,
        image_count,
        title,
        word_count,
    }
}
