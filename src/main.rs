#[macro_use]
extern crate lazy_static;

mod config;
mod converter;
mod domain;
mod writer;

use config::{get_config, Config};
use domain::{Content, File};
use std::fs;
use time::{format_description, format_description::FormatItem};

lazy_static! {
    static ref DATE_FORMAT: Vec<FormatItem<'static>> =
        format_description::parse("[month repr:short] [day] [year]").unwrap();
}

fn main() {
    let config: Config = get_config();
    let post_html =
        fs::read_to_string(&config.post_html_path).expect("Error reading post html base");
    let input_path = format!("{}/_posts", &config.site_directory);
    let files = get_files(&input_path);
    let posts = get_posts(files, &post_html);

    writer::write_posts(posts, &config.site_directory);
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

fn get_posts(files: Vec<File>, post_html: &str) -> Vec<Content> {
    files
        .iter()
        .map(|file| {
            let conversion = converter::convert(&file.markdown);
            let created = file.created.format(&DATE_FORMAT).unwrap();
            let read_time = get_read_time(conversion.image_count, conversion.word_count);

            let html = post_html
                .replace("{{content}}", &conversion.html)
                .replace("{{date}}", &created)
                .replace("{{read_time}}", &read_time)
                .replace("{{title}}", &conversion.title);

            Content {
                created: file.created.to_string(),
                filename: file.filename.to_string(),
                html,
                title: conversion.title,
            }
        })
        .collect()
}

fn get_read_time(image_count: usize, word_count: usize) -> String {
    let minutes = (word_count / 265) + ((image_count * 11) / 60);
    format!("{}m read time", minutes)
}
