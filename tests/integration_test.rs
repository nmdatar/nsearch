use nsearch::{
    index::builder,
    query::{parser, retrieval, scorer::{self, Bm25}},
};

#[test]
fn test_index_and_query_pipeline() {
    let dir = std::env::temp_dir();
    let pages   = dir.join("integration_pages.jsonl");
    let index_p = dir.join("integration_index.json");
    let store_p = dir.join("integration_store.json");

    std::fs::write(&pages, concat!(
        r#"{"url":"https://a.com","title":"Rust Lang","text":"rust memory safety ownership borrow checker"}"#, "\n",
        r#"{"url":"https://b.com","title":"Python Lang","text":"python dynamic typing garbage collection"}"#, "\n",
        r#"{"url":"https://c.com","title":"Rust Async","text":"rust async await tokio futures"}"#,
    )).unwrap();

    builder::build(&pages, &index_p, &store_p).unwrap();

    let index_json = std::fs::read_to_string(&index_p).unwrap();
    let store_json = std::fs::read_to_string(&store_p).unwrap();
    let index: nsearch::index::postings::Index = serde_json::from_str(&index_json).unwrap();
    let store: nsearch::index::store::DocStore = serde_json::from_str(&store_json).unwrap();

    let terms = parser::parse("rust memory");
    let candidates = retrieval::retrieve(&terms, &index);
    let scorer = Bm25 { k1: 1.2, b: 0.75 };
    let results = scorer::rank(candidates, &terms, &index, &scorer);

    // should return results
    assert!(!results.is_empty());

    // rust pages should rank above python page
    let top_url = &store.inner[&results[0].0].url;
    assert!(top_url.contains("a.com") || top_url.contains("c.com"),
        "expected a rust page to rank first, got {}", top_url);

    // python page should not be in top 2
    let top2_urls: Vec<&str> = results.iter().take(2)
        .map(|(id, _)| store.inner[id].url.as_str())
        .collect();
    assert!(!top2_urls.contains(&"https://b.com"));
}
