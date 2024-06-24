use std::{fs, io};
use std::path::Path;
use std::time::Duration;
use rand::Rng;
use reqwest::header;

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


// 创建一个包含默认请求头的HeaderMap
pub fn default_headers() -> header::HeaderMap {
    let mut headers = header::HeaderMap::new();

    headers.insert(header::USER_AGENT, header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36"));
    headers.insert(header::ACCEPT, header::HeaderValue::from_static("application/json, text/plain, */*"));
    headers.insert(header::ACCEPT_LANGUAGE, header::HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8,ja;q=0.7"));
    headers.insert(header::ACCEPT_ENCODING, header::HeaderValue::from_static("gzip, deflate, br"));

    headers
}