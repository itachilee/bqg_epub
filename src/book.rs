use std::cmp::Ordering;
use std::fs::{File, OpenOptions, read};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary, Result};
use futures::future;
use reqwest::Client;
use scraper::{Html, Selector};
use url::Url;
use crate::util::{check_and_create_directory, random_delay, replace_html_entities};

#[derive(Eq, Debug,Clone)]
pub struct Chapter {
    pub number: usize,
    pub href: String,
    pub title: String,
    pub content: String,
}

impl Chapter {
    pub fn new(href: &str, title: &str) -> Self {
        let number = href.split('.').next().unwrap_or("0").parse::<usize>().unwrap();
        Self {
            number,
            href: href.to_string(),
            title: title.to_string(),
            content: String::default(),
        }
    }


    pub fn update_content(&mut self,new_content: &str) ->Self{
        self.content = new_content.to_string();
        self.to_owned()
    }

    pub async fn scraper_chapter_content(&mut self, base_url: &str) -> Result<Self> {
        let base_url = Url::parse(base_url)?;
        let joined_url = base_url.join(&self.href)?;

        println!("now visited: {}", joined_url);
        let client = Client::new();
        let page = client.get(joined_url).send().await?.text().await?;
        let document = Html::parse_document(&page);
        let content_selector = Selector::parse("#content").unwrap();

        let content = match document.select(&content_selector).next() {
            Some(e) => {
                e.text().collect::<Vec<_>>().join("\r\n")
            }
            None => { "this chapter may have no content ".to_string() }
        };

        // self.content = replace_html_entities(&content);
        Ok(self.update_content(&replace_html_entities(&content)))
    }

    pub fn write_to_file(&self) ->Result<()> {
        let file_name = format!("books/{}.txt", self.href.split('.').next().unwrap_or("0").parse::<usize>().unwrap());
        let dir_path = Path::new(&file_name).parent().unwrap(); // Get the directory part of the file path

        check_and_create_directory(dir_path)?;
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_name).unwrap();

        file.write(self.content.as_bytes()).unwrap();
        Ok(())
    }
}

impl PartialEq for Chapter {
    fn eq(&self, other: &Self) -> bool {
        self.href == other.href
    }
}

impl PartialOrd for Chapter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Chapter {
    fn cmp(&self, other: &Self) -> Ordering {
        self.number.cmp(&other.number)
    }
}


#[derive(Debug)]
pub struct Book {
    pub title: String,
    pub homepage: String,
    pub intro: String,
    pub author: String,

    pub chapters: Vec<Chapter>,
}

impl Book {
    pub fn new(homepage: &str) -> Self {
        Self {
            title: String::default(),
            author: String::default(),
            intro: String::default(),
            chapters: Vec::new(),
            homepage: homepage.to_string(),
        }
    }


    pub async fn get_book_info(&mut self, client: &Client) -> epub_builder::Result<()> {
        let html = client.get(&self.homepage).send().await?.text().await?;
        println!("scraper homepage: {} start...", &self.homepage);

        // let mut chapters = vec![];
        let document = Html::parse_document(&html);
        let chapter_selector = Selector::parse("#list > dl > dd > a").unwrap();
        let author_selector = Selector::parse("#info > p:nth-child(2) > a").unwrap();
        let intro_selector = Selector::parse("#intro").unwrap();
        let title_selector = Selector::parse("#info > h1").unwrap();

        self.author = document.select(&author_selector).next().unwrap().text().collect::<Vec<_>>().join(" ");
        self.intro = document.select(&intro_selector).next().unwrap().text().collect::<Vec<_>>().join(" ");
        self.title = document.select(&title_selector).next().unwrap().text().collect::<Vec<_>>().join(" ");


        let mut  count =0;
        for element in document.select(&chapter_selector) {
            if let Some(href) = element.value().attr("href") {
                let text = element.text().collect::<Vec<_>>().join(" ");

                // if count >=3{
                //     break;
                // }
                // count+=1;
                println!("{} | {}", href, text);
                self.add_chapter(Chapter::new(href, &text));
            }
        }
        // chapters.sort();
        // self.chapters = chapters;

        println!("scraper homepage: {} done!", &self.homepage);
        Ok(())
    }

    pub fn add_chapter(&mut self, chapter: Chapter) {
        let exists = self.chapters.iter().any(|c| c.href == chapter.href);
        if !exists {
            self.chapters.push(chapter);
        }
    }


    pub async fn scraper_chapter(&mut self) -> Result<()> {
        self.chapters.sort();
        //
        //
        // // let client= Client::new();
        // for chapter in self.chapters.iter_mut() {
        //     println!("href: {}  | title: {}", chapter.href, chapter.title);
        //     let delay = random_delay();
        //     println!("Waiting for {} milliseconds before the next request...", delay.as_millis());
        //     tokio::time::sleep(delay).await;
        //     chapter.scraper_chapter_content(&self.homepage).await.unwrap();
        // }


        let mut handles = vec![];

        for chapter in self.chapters.iter_mut() {
            let homepage = self.homepage.clone();
            let mut chapter_ref = chapter.clone();
            // let mut chapter_ref = Arc::clone(chapter);
            let handle = tokio::spawn(async move {
                println!("href: {}  | title: {}", chapter_ref.href, chapter_ref.title);
                let delay = random_delay();
                println!("Waiting for {} milliseconds before the next request...", delay.as_millis());
                tokio::time::sleep(delay).await;
                chapter_ref.scraper_chapter_content(&homepage).await
            });
            handles.push(handle);
        }

        // 等待所有爬取任务完成
        let results = future::join_all(handles).await;
        for result in results {
            match result {
                Ok(res) => {
                    if let Ok(c) = res {

                        self.update_chapter_content(&c.href,&c.content);
                    }
                }
                Err(e) => {
                    eprintln!("Task join error: {:?}", e);
                }
            }
        }

        // 所有爬取工作完成后的进一步处理
        println!("All chapters fetched. Continue to the next step.");

        Ok(())
    }
    pub fn update_chapter_content(&mut self, href: &str, new_content: &str) {
        if let Some(chapter) = self.chapters.iter_mut().find(|chapter| chapter.href == href) {
            chapter.update_content(new_content);
        }
    }
    pub fn generate_epub(&self) -> Result<()> {
        // let mut output = Vec::<u8>::new();

        // todo: replace with real cover image
        let cover_image = File::open("cover.jpg").unwrap();


        let css = File::open("style/style.css")?;

        // let
        let mut output = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(format!("{}.epub", &self.title)).unwrap();

        // Create a new EpubBuilder using the zip library
        let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;
        builder
            // Set some metadata
            .metadata("author", &self.author)?
            .metadata("title", &self.title)?
            .metadata("description", &self.intro)?
            .add_cover_image("cover.jpg", cover_image, "image/jpg")?
            // Add a resource that is not part of the linear document structure
            // .add_resource("some_image.png", cover_image, "image/jpg")?
            .stylesheet(css)?
        ;

        for chapter in self.chapters.iter() {
            // builder.add_content(EpubContent::new(&chapter.href, &*chapter.content.as_bytes())
            //     .title(&chapter.title)
            //     .reftype(ReferenceType::TitlePage))?;


            let xhtml_content = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
            <!DOCTYPE html>
            <html xmlns="http://www.w3.org/1999/xhtml">
            <head>
                <title>{}</title>
                <link rel="stylesheet" type="text/css" href="style/styles.css" />
            </head>
            <body>
                <h1>{}</h1>
                <p>{}</p>
            </body>
            </html>"#,
                chapter.title, chapter.title, chapter.content
            );

            builder.add_content(
                EpubContent::new(&chapter.href, xhtml_content.as_bytes())
                    .title(&chapter.title)
                    .reftype(ReferenceType::Text),
            )?;
        }

        builder.inline_toc()
            // Finally, write the EPUB file to a writer. It could be a `Vec<u8>`, a file,
            // `stdout` or whatever you like, it just needs to implement the `std::io::Write` trait.
            .generate(&mut output)?;


        Ok(())
    }


    pub async fn start_scrape(&mut self)-> Result<()>{

        let client = Client::builder()
            .build()?;
        self.get_book_info(&client).await.unwrap();

        self.scraper_chapter().await?;

        // println!("{:?}",self);
        self.generate_epub()?;
        Ok(())
    }

    pub  fn print_info(&self) {
        println!("{:?}", self);
    }
}
