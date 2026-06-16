pub mod fetcher;
pub mod frontier;

use frontier::Frontier;
use fetcher::{FetchResult, fetch};

use std::io::Write;
use std::path::Path;

pub async fn crawl(seed: String, limit: usize, output: &Path) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; nsearch/0.1)")
        .build()?;
    let mut frontier = Frontier::new();
    let mut file = std::fs::File::create(output)?;

    frontier.push(seed);
    
    let mut count = 0;

    while let Some(url) = frontier.pop() {
        if count >= limit {
            break;
        }
        
        let parsed_url = match url::Url::parse(&url) {
            Ok(u) => u,
            Err(_) => continue,
        };

        let result = match fetch(&client, &parsed_url).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("skipping {}: {}", url, e);
                count += 1;
                continue;
            }
        };

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

        let _mock1 = server.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(format!(r#"
                <html>
                    <head><title>Page 1</title></head>
                    <body><a href="{}/page2">Page 2</a></body>
                </html>
            "#, server.url()))
            .create_async()
            .await;

        let _mock2 = server.mock("GET", "/page2")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(r#"<html><head><title>Page 2</title></head><body>No links</body></html>"#)
            .create_async()
            .await;

        let path = std::env::temp_dir().join("test_crawl_limit.jsonl");

        crawl(server.url(), 1, &path).await.unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents.lines().count(), 1, "should only crawl 1 page");

        std::fs::remove_file(&path).unwrap();
    }

    #[tokio::test]
    async fn test_crawl_follows_child_links() {
        let mut server = mockito::Server::new_async().await;

        let _mock1 = server.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(format!(r#"
                <html>
                    <head><title>Page 1</title></head>
                    <body><a href="{}/page2">Page 2</a></body>
                </html>
            "#, server.url()))
            .create_async()
            .await;

        let _mock2 = server.mock("GET", "/page2")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(r#"<html><head><title>Page 2</title></head><body>No links</body></html>"#)
            .create_async()
            .await;

        let path = std::env::temp_dir().join("test_crawl_links.jsonl");

        crawl(server.url(), 2, &path).await.unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents.lines().count(), 2, "should have crawled both pages");
        assert!(contents.contains("Page 1"));
        assert!(contents.contains("Page 2"));

        std::fs::remove_file(&path).unwrap();
    }
}