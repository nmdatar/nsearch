use scraper::{Html, Selector};

#[derive(serde::Serialize)]
pub struct FetchResult {
    pub url: String,
    pub title: String,
    pub text: String,
    pub links: Vec<String>,
}

pub async fn fetch(client: &reqwest::Client, url: &url::Url) -> anyhow::Result<FetchResult> {
    let html = client.get(url.as_str()).send().await?.text().await?;
    let document = Html::parse_document(&html);
    let title_sel = Selector::parse("title").unwrap();
    let title = document
    .select(&title_sel)
    .next()                          // first <title> element
    .map(|el| el.text().collect())   // get its text
    .unwrap_or_default();            // fall back to empty string

    let link_sel = Selector::parse("a[href]").unwrap();
    let links: Vec<String> = document
    .select(&link_sel)
    .filter_map(|el| el.attr("href"))
    .filter_map(|href| url.join(href).ok())  // resolve relative URLs against base
    .map(|u| u.to_string())
    .collect();

    let body_sel = Selector::parse("body").unwrap();
    let text = document
    .select(&body_sel)
    .next()
    .map(|el| el.text().collect())
    .unwrap_or_default();

    Ok(FetchResult {
        url: url.to_string(),
        title,
        text,
        links
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_extracts_title_and_links() {
        let mut server = mockito::Server::new_async().await;

        let mock = server.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(r#"
                <html>
                    <head><title>Test Page</title></head>
                    <body>
                        <p>Hello world</p>
                        <a href="/about">About</a>
                        <a href="/contact">Contact</a>
                    </body>
                </html>
            "#)
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let url = url::Url::parse(&server.url()).unwrap();
        let result = fetch(&client, &url).await.unwrap();

        assert_eq!(result.title, "Test Page");
        assert!(result.text.contains("Hello world"));
        assert_eq!(result.links.len(), 2);
        assert!(result.links.iter().any(|l| l.ends_with("/about")));

        mock.assert_async().await;
    }
}