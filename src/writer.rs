use crate::domain::Content;
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
