use serde::Deserialize;

use crate::index::postings::DocId;
use std::{collections::HashMap};

#[derive(serde::Serialize, Deserialize)]
pub struct DocMeta {
    pub url: String,
    pub title: String,
}
#[derive(serde::Serialize, Deserialize)]
pub struct DocStore {
    pub inner: HashMap<DocId, DocMeta>,
}

impl DocStore {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
}