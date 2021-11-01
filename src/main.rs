#[macro_use]
extern crate lazy_static;

mod config;
mod converter;
mod domain;
mod writer;

use config::{get_config, Config};
use domain::{Content, File, PostMaps};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use time::{format_description, format_description::FormatItem};

lazy_static! {
    static ref DATE_FORMAT: Vec<FormatItem<'static>> =
        format_description::parse("[month repr:short] [day] [year]").unwrap();
    static ref TAGS_REGEX: Regex = Regex::new(r"\[tags\]: # \((.+)\)").unwrap();
}

fn main() {
    let config: Config = get_config();
    let post_html = fs::read_to_string(&config.post_html_path).expect("Error reading post html");
    let tag_html = fs::read_to_string(&config.tag_html_path).expect("Error reading tag html");
    let input_path = format!("{}/_posts", &config.site_directory);
    let files = get_files(&input_path);
    let post_maps = get_post_maps(&files, &post_html, &config.site_url);
    let posts = get_posts_with_related(&post_maps, &config.site_url);

    writer::write_posts(posts, &config.site_directory);
    writer::write_tag_pages(&post_maps, &config, &tag_html);
    writer::write_rss_feed(&post_maps, &config);
}

fn get_files(input_path: &str) -> Vec<File> {
    let file_paths = fs::read_dir(input_path).expect("Error reading input paths");

    let mut files: Vec<File> = file_paths
        .enumerate()
        .map(|(_, path)| {
            let path = path.unwrap().path().display().to_string();
            let filename = path.rsplit('/').next().unwrap().replace(".md", "");
            let created = fs::metadata(&path).unwrap().created().unwrap().into();
            let markdown = fs::read_to_string(&path).expect("Unable to read file");

            File {
                created,
                filename,
                markdown,
            }
        })
        .collect();

    // Sort here so that OffsetDateTimes don't need to be sorted for each collection later
    files.sort_by(|a, b| b.created.cmp(&a.created));
    files
}

fn get_post_maps(post_files: &[File], post_html: &str, base_site_url: &str) -> PostMaps {
    let mut tag_posts = HashMap::new();

    let posts: HashMap<String, Content> = post_files
        .iter()
        .map(|post_file| {
            let tags = match TAGS_REGEX.captures(&post_file.markdown) {
                Some(tags) => tags[1]
                    .split(',')
                    .map(str::trim)
                    .map(str::to_owned)
                    .collect(),
                _ => Vec::new(),
            };

            tags.iter().for_each(|tag| {
                tag_posts
                    .entry(tag.to_string())
                    .or_insert_with(Vec::new)
                    .push(post_file.filename.to_string());
            });

            let created = post_file.created.format(&DATE_FORMAT).unwrap();

            let tags_html = tags
                .iter()
                .map(|tag| {
                    let url = format!("{}/tags/{}", base_site_url, tag);
                    format!("<a href=\"{}\">{}</a>", url, tag)
                })
                .collect::<Vec<String>>()
                .join(", ");

            let conversion = converter::convert(&post_file.markdown);
            let title = conversion.title;
            let read_time = get_read_time(conversion.image_count, conversion.word_count);

            let html = post_html
                .replace("{{content}}", &conversion.html)
                .replace("{{date}}", &created)
                .replace("{{read_time}}", &read_time)
                .replace("{{title}}", &title)
                .replace("{{tags}}", &tags_html);

            (
                post_file.filename.to_string(),
                Content {
                    created,
                    filename: post_file.filename.to_string(),
                    html,
                    tags,
                    title,
                },
            )
        })
        .collect();

    PostMaps { posts, tag_posts }
}

fn get_posts_with_related(post_maps: &PostMaps, base_site_url: &str) -> Vec<Content> {
    post_maps
        .posts
        .iter()
        .map(|(filename, post)| {
            let mut scored: HashMap<String, i32> = HashMap::new();

            post.tags.iter().for_each(|tag| {
                post_maps.tag_posts[tag].iter().for_each(|path| {
                    let count = scored.entry(path.to_string()).or_insert_with(|| 0);
                    *count += 1;
                });
            });

            let mut scored = scored.into_iter().collect::<Vec<(String, i32)>>();
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            // Don't include current post in related posts
            let related_html = scored.iter().filter(|a| &a.0 != filename).take(3).fold(
                String::new(),
                |acc, score| {
                    let filename = score.0.to_string();
                    let tmp = &post_maps.posts[&filename];
                    let url = format!("{}/posts/{}.html", base_site_url, filename);
                    format!("{}<a href=\"{}\">{}</a>", acc, url, tmp.title)
                },
            );

            Content {
                created: post.created.to_string(),
                filename: filename.to_string(),
                html: post.html.replace("{{related}}", &related_html),
                tags: post.tags.clone(),
                title: post.title.to_string(),
            }
        })
        .collect()
}

fn get_read_time(image_count: usize, word_count: usize) -> String {
    let minutes = (word_count / 265) + ((image_count * 11) / 60);
    format!("{}m read time", minutes)
}
