use std::any::Any;
use epub_builder::Result;
use clap::Parser;
use bqg_epub::bqg::book::Book;
use bqg_epub::hitokoto::sentence::random_sentence;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(next_line_help = true)]
struct Args {
    /// base_url e.g: https://www.xbiqugew.com/book/53099/
    #[arg(short, long)]
    url: Option<String>,
    /// 输出每日一言
    #[arg( long)]
    hitokoto: Option<String>,
}


#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    run(&args).await?;
    Ok(())
}


async fn run(args: &Args) -> Result<()> {
    if args.url.is_some() {
        let mut book = Book::new(&args.url.clone().unwrap());
        book.start_scrape().await?;
    } else if args.hitokoto.is_some() {
        random_sentence(&args.hitokoto.clone().unwrap());
    }
    Ok(())
}



trait MyTrait {
    fn trait_hello(&self);
    fn as_any(&self) -> &dyn Any;
}
struct MyStruct1;

impl MyStruct1 {
    fn struct_hello(&self) {
        println!("Hello, world! from MyStruct1");
    }
}

struct MyStruct2;

impl crate::MyStruct2 {
    fn struct_hello(&self) {
        println!("Hello, world! from MyStruct1");
    }
}
impl MyTrait for MyStruct1 {
    fn trait_hello(&self) {
        self.struct_hello();
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl MyTrait for MyStruct2 {
    fn trait_hello(&self) {
        self.struct_hello();
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}