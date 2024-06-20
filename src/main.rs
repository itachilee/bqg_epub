use std::fs::File;
use epub_builder::Result;








use clap::Parser;
use reqwest::{Client};


use bqg_epub::book::Book;
use bqg_epub::util::random_delay;


// const BASE_URL: &str = "https://www.xbiqugew.com/book/53099/";
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(next_line_help = true)]
struct Args {
    /// base_url
    #[arg(short, long)]
    url: String,
}


#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();


    let client = Client::builder()
        .build()?;

    let mut book = Book::new(&args.url);
    book.get_book_info(&client).await.unwrap();

    book.scraper_chapter(&client).await?;



    book.generate_epub()?;

    println!("{:?}", book);
    Ok(())
}


