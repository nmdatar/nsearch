use std::{collections::HashMap, path::Path};
use crate::{analysis::tokenizer::analyze, index::{postings, store}};
use std::io::BufRead;

#[derive(serde::Deserialize)]
struct PageRecord {
    url: String,
    title: String,
    text: String,
}

pub fn build(input: &Path, index_output: &Path, store_output: &Path) -> anyhow::Result<()> {
    let file = std::fs::File::open(input)?;
    let reader = std::io::BufReader::new(file);
    let mut index = postings::Index::new();
    let mut doc_store: store::DocStore = store::DocStore::new();
    let mut doc_id: u32 = 0;
    let mut total_tokens: u64 = 0;
    
    for line in reader.lines() {
        let line = line?;
        let page: PageRecord = serde_json::from_str(&line)?;
        let mut term_freqs: HashMap<String, u32> = HashMap::new();
        let tokens = analyze(&page.text);

        doc_store.inner.insert(doc_id,store::DocMeta {url: page.url, title: page.title});

        for token in &tokens {
            *term_freqs.entry(token.clone()).or_insert(0) += 1;
        }

        for (term, freq) in term_freqs {
            index.postings
            .entry(term)
            .or_insert(Vec::new())
            .push(postings::Posting {doc_id, term_freq: freq});
        } 

        index.doc_lengths.insert(doc_id, tokens.len() as u32);
        doc_id += 1;
        total_tokens += tokens.len() as u64;
    }

    index.num_docs = doc_id;
    if doc_id > 0 {
        index.avg_dl = total_tokens as f64 / doc_id as f64;
    }

    let index_file = std::fs::File::create(index_output)?;
    let store_file = std::fs::File::create(store_output)?;
    serde_json::to_writer(index_file, &index)?;
    serde_json::to_writer(store_file, &doc_store)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_temp_pages(name: &str, lines: &[&str]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(name);
        std::fs::write(&path, lines.join("\n")).unwrap();
        path
    }

    #[test]
    fn test_build_indexes_terms() {
        let input = write_temp_pages("pages1.jsonl", &[
            r#"{"url":"https://a.com","title":"A","text":"rust async rust"}"#,
        ]);
        let index_out = std::env::temp_dir().join("index1.json");
        let store_out = std::env::temp_dir().join("store1.json");

        build(&input, &index_out, &store_out).unwrap();

        let index: postings::Index = serde_json::from_str(
            &std::fs::read_to_string(&index_out).unwrap()
        ).unwrap();

        assert!(index.postings.contains_key("rust"));
        assert_eq!(index.postings["rust"][0].term_freq, 2);
        assert_eq!(index.num_docs, 1);
    }

    #[test]
    fn test_build_avg_dl() {
        let input = write_temp_pages("pages2.jsonl", &[
            r#"{"url":"https://a.com","title":"A","text":"one two three"}"#,
            r#"{"url":"https://b.com","title":"B","text":"one two three four five"}"#,
        ]);
        let index_out = std::env::temp_dir().join("index2.json");
        let store_out = std::env::temp_dir().join("store2.json");

        build(&input, &index_out, &store_out).unwrap();

        let index: postings::Index = serde_json::from_str(
            &std::fs::read_to_string(&index_out).unwrap()
        ).unwrap();

        assert_eq!(index.avg_dl, 4.0);
        assert_eq!(index.num_docs, 2);
    }
}