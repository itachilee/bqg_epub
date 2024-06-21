use std::cmp::Ordering;
use std::fs::{File, OpenOptions, read};
use std::io::Read;
use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary, Result};
use reqwest::Client;
use scraper::{Html, Selector};
use url::Url;
use crate::util::{random_delay, replace_html_entities};

#[derive(Eq, Debug)]
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

    pub async fn scraper_chapter_content(&mut self, base_url: &str, client: &Client) -> Result<()> {
        let base_url = Url::parse(base_url)?;
        let joined_url = base_url.join(&self.href)?;

        println!("now visited: {}", joined_url);

        let page = client.get(joined_url).send().await?.text().await?;
        let document = Html::parse_document(&page);
        let content_selector = Selector::parse("#content").unwrap();

        let content = match document.select(&content_selector).next() {
            Some(e) => {
                e.text().collect::<Vec<_>>().join("\r\n")
            }
            None => { "this chapter may have no content ".to_string() }
        };

        // let file_name = format!("books/{}.txt", self.href.split('.').next().unwrap_or("0").parse::<usize>().unwrap());
        // let dir_path = Path::new(&file_name).parent().unwrap(); // Get the directory part of the file path

        // check_and_create_directory(dir_path)?;
        // let mut file = OpenOptions::new()
        //     .read(true)
        //     .write(true)
        //     .create(true)
        //     .open(file_name).unwrap();

        // file.write(cleaned.as_bytes()).unwrap();

        self.content = replace_html_entities(&content);
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


        for element in document.select(&chapter_selector) {
            if let Some(href) = element.value().attr("href") {
                let text = element.text().collect::<Vec<_>>().join(" ");


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


    pub async fn scraper_chapter(&mut self, client: &Client) -> Result<()> {
        self.chapters.sort();

        for chapter in self.chapters.iter_mut() {
            println!("href: {}  | title: {}", chapter.href, chapter.title);
            let delay = random_delay();
            println!("Waiting for {} milliseconds before the next request...", delay.as_millis());
            tokio::time::sleep(delay).await;
            chapter.scraper_chapter_content(&self.homepage, &client).await.unwrap();
        }

        Ok(())
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

        self.scraper_chapter(&client).await?;

        self.generate_epub()?;
        Ok(())
    }

    pub  fn print_info(&self) {
        println!("{:?}", self);
    }
}
