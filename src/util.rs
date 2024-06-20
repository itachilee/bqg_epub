use std::{fs, io};
use std::path::Path;
use std::time::Duration;
use rand::Rng;

pub fn check_and_create_directory(dir_path: &Path) -> io::Result<()> {
    if !dir_path.exists() {
        println!("Directory does not exist. Creating directory: {:?}", dir_path);
        fs::create_dir_all(dir_path)?; // Create the directory and any missing parent directories
    } else {
        println!("Directory already exists: {:?}", dir_path);
    }
    Ok(())
}

pub fn random_delay() -> Duration {
    let mut rng = rand::thread_rng();
    let millis = rng.gen_range(500..2000); // Random delay between 500ms and 2000ms
    Duration::from_millis(millis)
}

pub fn replace_html_entities(s: &str) -> String {
    s.replace("&nbsp;", "")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
    // .replace(" "," ")
    // Add more replacements as needed
}