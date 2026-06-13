pub mod fetcher;
pub mod frontier;

use frontier::Frontier;
use fetcher::{FetchResult, fetch};

use std::io::Write;
use std::path::Path;

pub async fn crawl(seed: String, limit: usize, output: &Path) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let mut frontier = Frontier::new();
    let mut file = std::fs::File::create(output)?;

    frontier.push(seed);
    
    let mut count = 0;

    while let Some(url) = frontier.pop() {
        if count >= limit {
            break;
        }
        
        let parsed_url = url::Url::parse(&url)?;
        let result = fetch(&client, &parsed_url).await?;

        write_result(&mut file, &result)?;

        for child_link in result.links {
            frontier.push(child_link);
        }

        count +=1;
    }

    Ok(())
}

fn write_result(file: &mut std::fs::File, result: &FetchResult) -> anyhow::Result<()> {
    serde_json::to_writer(&mut *file, result)?;
    writeln!(file)?;    
    Ok(())
}