use clap::{Parser, Subcommand};
use nsearch::{crawler, index::{builder}, query::{parser, retrieval, scorer}};
use std::path::Path;

#[derive(Parser)]
#[command(name = "nsearch", about = "A search engine")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Crawl {
        #[arg(long)]
        seed: String,
        #[arg(long, default_value_t = 100)]
        limit: usize,
        #[arg(long, default_value = "data/pages.jsonl")]
        output: String,
    },    
    Index {
        #[arg(long, default_value = "data/pages.jsonl")]
        pages_path: String,
        #[arg(long, default_value = "data/index.json")]
        index_output: String,
        #[arg(long, default_value = "data/store.json")]
        store_output: String,
    },
    Query {
        terms: String,
        #[arg(long, default_value = "data/index.json")]
        index_path: String,
        #[arg(long, default_value = "data/store.json")]
        store_path: String,
        #[arg(long, default_value = "bm25")]
        scorer: ScorerArg,
        #[arg(long, default_value_t = 1.2)]
        k1: f64,
        #[arg(long, default_value_t = 0.75)]
        b: f64,
    },
}

#[derive(clap::ValueEnum, Clone)]
  enum ScorerArg {
      Bm25,
      Tfidf,
  }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Crawl { seed, limit, output} => {
            crawler::crawl(seed, limit, Path::new(&output)).await?;
        }
        Command::Index { pages_path, index_output, store_output } => {
            builder::build(Path::new(&pages_path), Path::new(&index_output), Path::new(&store_output))?;
        }
        Command::Query { terms, index_path, store_path, scorer, k1, b } => {
            let parsed_terms = parser::parse(&terms);
            let index = &std::fs::read_to_string(&index_path)?;
            let store = &std::fs::read_to_string(&store_path)?;
            let index: nsearch::index::postings::Index = serde_json::from_str(index)?;
            let store: nsearch::index::store::DocStore = serde_json::from_str(store)?;
            let relevant_docs = retrieval::retrieve(&parsed_terms, &index);
            let scorer: Box<dyn scorer::Scorer> = match scorer {
                ScorerArg::Tfidf => Box::new(scorer::TfIdf),
                ScorerArg::Bm25 => Box::new(scorer::Bm25 { k1, b })
            };
            let results = scorer::rank(relevant_docs, &parsed_terms, &index, &*scorer);

            for (doc_id, score) in results.iter().take(10) {
                if let Some(metadata) = store.inner.get(doc_id) {
                    println!("{:4} {} {}", score, metadata.title, metadata.url);
                }
            }
        }
    }
    
    Ok(())
}