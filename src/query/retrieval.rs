use std::collections::HashSet;

use crate::index::postings::{DocId, Index};

pub fn retrieve(terms: &[String], index: &Index) -> Vec<DocId> {
    let mut set: HashSet<DocId> = HashSet::new();
    for term in terms {
        if let Some(postings) = index.postings.get(term) {
            set.extend(postings.iter().map(|p| p.doc_id));
        }
    }
    set.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::postings::{Index, Posting};

    fn make_index(postings: Vec<(&str, Vec<(u32, u32)>)>) -> Index {
        let mut index = Index::new();
        for (term, docs) in postings {
            index.postings.insert(
                term.to_string(),
                docs.into_iter()
                    .map(|(doc_id, term_freq)| Posting { doc_id, term_freq })
                    .collect(),
            );
        }
        index
    }

    #[test]
    fn test_retrieve_returns_union_of_matching_docs() {
        let index = make_index(vec![
            ("rust", vec![(0, 2), (2, 1)]),
            ("async", vec![(0, 1), (3, 1)]),
        ]);

        let terms = vec!["rust".to_string(), "async".to_string()];
        let mut result = retrieve(&terms, &index);
        result.sort();

        assert_eq!(result, vec![0, 2, 3]);
    }
}
