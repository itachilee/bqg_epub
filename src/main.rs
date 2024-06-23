use epub_builder::Result;
use clap::Parser;
use bqg_epub::book::Book;



#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(next_line_help = true)]
struct Args {
    /// base_url e.g: https://www.xbiqugew.com/book/53099/
    #[arg(short, long)]
    url: String,
}


#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut book = Book::new(&args.url);
    book.start_scrape().await?;
    Ok(())
}


