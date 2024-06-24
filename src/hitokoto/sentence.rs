use std::fs;
use std::fs::File;
use serde::{Deserialize, Serialize};

use serde_json::Value;
use std::io::{BufReader, Result};
use std::path::{Path, PathBuf};
use rand::Rng;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sentence {
    pub id: i64,
    pub uuid: String,
    pub hitokoto: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub from: String,
    #[serde(rename = "from_who")]
    pub from_who: Value,
    pub creator: String,
    #[serde(rename = "creator_uid")]
    pub creator_uid: i64,
    pub reviewer: i64,
    #[serde(rename = "commit_from")]
    pub commit_from: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    pub length: i64,
}


pub struct Category {
    pub category: String,
    pub sentence: Vec<Sentence>,
}


impl Category {
    fn new(category: &str) -> Self {
        Self {
            category: category.to_string(),
            sentence: vec![],
        }
    }


}

pub fn read_sentences() -> Result<Vec<Category>> {
    let current_dir = "sentences";
    let entries = fs::read_dir(current_dir)?
        .filter_map(Result::ok) // 过滤掉可能的错误
        .map(|entry| entry.path())
        .filter(|path| path.is_file()); // 仅保留文件

    let mut categories = vec![];
    for path in entries {
        if path.is_file() {
            let byte_content = File::open(&path)?;
            let reader = BufReader::new(byte_content);
            let sentence: Vec<Sentence> = serde_json::from_reader(reader)?;
            categories.push(Category {
                category: get_file_stem(&path),
                sentence,
            });
        }
    }

    Ok(categories)
}
fn get_file_stem(path: &PathBuf) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|s| s.to_string())
        .unwrap_or("a".to_string())
}


pub fn random_sentence(category: &str){
    let categories = read_sentences().unwrap();
    let c =categories.iter().filter(|c| c.category == category).last();
    match c {
        Some(c)=>{
            let mut rng = rand::thread_rng();
            let idx =rng.gen_range(0..c.sentence.len());
            let sentence = c.sentence.get(idx).unwrap();
            println!("today's hitokoto:\r\n{:?}",sentence);
        }
        None=>{
            println!("not found category: {}",category);
        }
    }
}
#[cfg(test)]
mod test {
    use std::fs;
    use std::fs::File;
    use std::io::{BufReader, Read};
    use super::*;

    #[test]
    fn test_files() -> Result<()> {
        let current_dir = "sentences";
        let entries = fs::read_dir(current_dir)?
            .filter_map(Result::ok) // 过滤掉可能的错误
            .map(|entry| entry.path())
            .filter(|path| path.is_file()); // 仅保留文件

        let mut i = 0;
        for path in entries {
            if let Some(file_name) = path.file_name() {
                // if let Some(name) = file_name.to_str() {
                //     println!("{}", name);
                // }

                if i == 0 {
                    let byte_content = File::open(path)?;
                    let reader = BufReader::new(byte_content);
                    let book: Vec<Sentence> = serde_json::from_reader(reader)?;
                    println!("{:?}", book.first().ok_or("no more"));
                }
                i += 1;
            }
        }

        assert_eq!(1, 1);
        Ok(())
    }

    #[test]
    fn test_read_files(){

        let res =read_sentences();
    }
}