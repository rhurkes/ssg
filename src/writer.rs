use crate::config::Config;
use crate::domain::{Content, PostMaps};
use std::fs;

pub fn write_posts(posts: Vec<Content>, site_directory: &str) {
    let posts_directory = format!("{}/posts/", site_directory);
    fs::remove_dir_all(&posts_directory).unwrap_or(());
    fs::create_dir(&posts_directory).unwrap();

    posts.iter().for_each(|content| {
        let new_path = format!("{}{}.html", posts_directory, content.filename);
        fs::write(new_path, &content.html).expect("Unable to write file");
    });
}

pub fn write_tag_pages(post_maps: &PostMaps, config: &Config, base_tag_html: &str) {
    let tags_directory = format!("{}/tags/", &config.site_directory);
    fs::remove_dir_all(&tags_directory).unwrap_or(());
    fs::create_dir(&tags_directory).unwrap();

    let all_html = post_maps
        .posts
        .iter()
        .fold(String::new(), |acc, (_, post)| {
            let url = format!("{}/posts/{}.html", &config.site_url, post.filename);
            format!(
                "{}<li><a href=\"{}\">{}</a> {}</li>",
                acc, url, post.title, post.created
            )
        });

    let new_path = format!("{}{}.html", tags_directory, "all");
    let all_html = base_tag_html
        .replace("{{title}}", "Posts tagged \"all\":")
        .replace("{{content}}", &all_html);
    fs::write(new_path, &all_html).expect("Unable to write file");

    post_maps.tag_posts.iter().for_each(|(tag, posts)| {
        let tag_html = posts.iter().fold(String::new(), |acc, filename| {
            let post = &post_maps.posts[filename];
            let url = format!("{}/posts/{}.html", &config.site_url, post.filename);
            format!(
                "{}<li><a href=\"{}\">{}</a> {}</li>",
                acc, url, post.title, post.created
            )
        });

        let new_path = format!("{}{}.html", tags_directory, tag);
        let tag_html = base_tag_html
            .replace("{{title}}", &format!("Posts tagged \"{}\":", tag))
            .replace("{{content}}", &tag_html);
        fs::write(new_path, &tag_html).expect("Unable to write file");
    });
}

pub fn write_rss_feed(post_maps: &PostMaps, config: &Config) {
    let feed = r#"<?xml version="1.0" encoding="UTF-8" ?>
        <rss version="2.0">
            <channel>
                <title>sigtor</title>
                <link>https://sigtor.org</link>
                <description>Writings about Software, Weather, etc.</description>
                {{items}}
            </channel>
        </rss>"#;

    let base_item = r#"<item>
        <title>{{title}}</title>
        <link>{{link}}</link>
        <pubDate>{{created}}</pubDate>
    </item>"#;

    let item_html = post_maps
        .posts
        .iter()
        .take(8)
        .fold(String::new(), |acc, (_, post)| {
            let url = format!("{}/posts/{}.html", &config.site_url, post.filename);
            let item = base_item
                .replace("{{title}}", &post.title)
                .replace("{{link}}", &url)
                .replace("{{created}}", &post.created);
            format!("{}{}", acc, item)
        });

    let feed = feed.replace("{{items}}", &item_html);
    let new_path = format!("{}/feed.xml", &config.site_directory);
    fs::write(new_path, &feed).expect("Unable to write file");
}
