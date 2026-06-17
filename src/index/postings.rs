use std::collections::HashMap;

pub type DocId = u32;
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Posting {
    pub doc_id: DocId,
    pub term_freq: u32,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Index {
    pub postings: HashMap<String, Vec<Posting>>,
    pub doc_lengths: HashMap<DocId, u32>,
    pub avg_dl: f64,
    pub num_docs: u32,
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

impl Index {
    pub fn new() -> Self {
        Self {
            postings: HashMap::new(),
            doc_lengths: HashMap::new(),
            avg_dl: 0.0,
            num_docs: 0,
        }
    }
}
