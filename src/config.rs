use serde_derive::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub post_html_path: String,
    pub site_url: String,
    pub site_directory: String,
}

pub fn get_config() -> Config {
    let home_path = home::home_dir().expect("Unable to find user's home directory");
    let home_path = home_path.to_str().unwrap();
    let config_path = format!("{}/{}", home_path, ".config/ssg/config.toml");
    let config_contents = fs::read_to_string(config_path).expect("Unable to read config file");
    let config_absolute_paths = config_contents.replace("~", home_path);
    let config: Config = toml::from_str(&config_absolute_paths).expect("Error reading config");
    config
}
