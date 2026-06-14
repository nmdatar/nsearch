use clap::Parser;
use nsearch::crawler;
use std::path::Path;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    crawler::crawl(args.seed, args.limit, Path::new(&args.output)).await?;
    Ok(())
}

#[derive(Parser)]
#[command(name = "nsearch", about = "A search engine")]
struct Args {
    #[arg(long, help = "Seed URL to start crawling from")]
    seed: String,

    #[arg(long, default_value_t = 100, help = "Max pages to crawl")]
    limit: usize,

    #[arg(long, default_value = "data/pages.jsonl", help = "Output file")]
    output: String,
}