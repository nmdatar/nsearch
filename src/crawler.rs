pub mod fetcher;
pub mod frontier;

use fetcher::{FetchResult, fetch};
use frontier::Frontier;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};

const PERMITS: usize = 10;

pub async fn crawl(seed: String, limit: usize, output: &Path) -> anyhow::Result<()> {
    let client = Arc::new(
        reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (compatible; nsearch/0.1)")
            .build()?,
    );
    let mut frontier = Frontier::new();
    let file = Arc::new(Mutex::new(std::fs::File::create(output)?));
    let semaphore = Arc::new(Semaphore::new(PERMITS));
    frontier.push(seed);
    let mut count = 0;
    let mut tasks: JoinSet<anyhow::Result<Vec<String>>> = JoinSet::new();

    loop {
        while count < limit {
            let url = frontier.pop();
            let Some(url) = url else {
                break;
            };
            let client = client.clone();
            let file = file.clone();
            let permit = semaphore.clone().acquire_owned().await?;
            count += 1;

            tasks.spawn(async move {
                let _permit = permit;
                let parsed_url = match url::Url::parse(&url) {
                    Ok(u) => u,
                    Err(_) => return Ok(vec![]),
                };

                let result = match fetch(&client, &parsed_url).await {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("skipping {}: {}", url, e);
                        return Ok(vec![]);
                    }
                };

                {
                    let mut f = file.lock().unwrap();
                    write_result(&mut f, &result)?;
                }

                Ok(result.links)
            });
        }

        if tasks.is_empty() {
            break;
        }

        match tasks.join_next().await.unwrap() {
            Ok(Ok(links)) => {
                for link in links {
                    frontier.push(link);
                }
            }
            Ok(Err(e)) => eprint!("fetch error: {}", e),
            Err(e) => eprintln!("task panic: {}", e),
        }
    }

    Ok(())
}

fn write_result(file: &mut std::fs::File, result: &FetchResult) -> anyhow::Result<()> {
    serde_json::to_writer(&mut *file, result)?;
    writeln!(file)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_result_writes_json_line() {
        let path = std::env::temp_dir().join("test_write_result.jsonl");
        let mut file = std::fs::File::create(&path).unwrap();

        let result = FetchResult {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            text: "Hello world".to_string(),
            links: vec![],
        };

        write_result(&mut file, &result).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("example.com"));
        assert!(contents.contains("Example"));
        assert!(contents.ends_with('\n'));
        std::fs::remove_file(&path).unwrap();
    }

    #[tokio::test]
    async fn test_crawl_stops_at_limit() {
        let mut server = mockito::Server::new_async().await;

        let _m1 = server.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(format!(
                r#"<html><head><title>Page 1</title></head><body><a href="{}/page2">Page 2</a></body></html>"#,
                server.url()
            ))
            .create_async().await;

        let _m2 = server
            .mock("GET", "/page2")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(r#"<html><head><title>Page 2</title></head><body></body></html>"#)
            .create_async()
            .await;

        let path = std::env::temp_dir().join("test_crawl_limit.jsonl");
        crawl(server.url(), 1, &path).await.unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents.lines().count(), 1);
        std::fs::remove_file(&path).unwrap();
    }

    #[tokio::test]
    async fn test_crawl_follows_links() {
        let mut server = mockito::Server::new_async().await;

        let _m1 = server.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(format!(
                r#"<html><head><title>Page 1</title></head><body><a href="{}/page2">Page 2</a></body></html>"#,
                server.url()
            ))
            .create_async().await;

        let _m2 = server
            .mock("GET", "/page2")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(r#"<html><head><title>Page 2</title></head><body></body></html>"#)
            .create_async()
            .await;

        let path = std::env::temp_dir().join("test_crawl_links.jsonl");
        crawl(server.url(), 2, &path).await.unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents.lines().count(), 2);
        assert!(contents.contains("Page 1"));
        assert!(contents.contains("Page 2"));
        std::fs::remove_file(&path).unwrap();
    }

    #[tokio::test]
    async fn test_crawl_deduplicates_urls() {
        let mut server = mockito::Server::new_async().await;

        // page 1 links to page 2 twice — should only crawl page 2 once
        let _m1 = server
            .mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(format!(
                r#"<html><head><title>Page 1</title></head><body>
                    <a href="{0}/page2">Link A</a>
                    <a href="{0}/page2">Link B</a>
                </body></html>"#,
                server.url()
            ))
            .create_async()
            .await;

        let _m2 = server
            .mock("GET", "/page2")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(r#"<html><head><title>Page 2</title></head><body></body></html>"#)
            .expect(1) // must be called exactly once
            .create_async()
            .await;

        let path = std::env::temp_dir().join("test_crawl_dedup.jsonl");
        crawl(server.url(), 10, &path).await.unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents.lines().count(), 2);
        _m2.assert_async().await;
        std::fs::remove_file(&path).unwrap();
    }
}
