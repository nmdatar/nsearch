use crate::index::postings::{DocId, Index};

pub trait Scorer {
    fn score(
        &self,
        term_freq: u32,
        doc_length: u32,
        docs_containing_term: usize,
        num_docs: u32,
        avg_dl: f64,
    ) -> f64;
}

pub struct TfIdf;

pub struct Bm25 {
    pub k1: f64,
    pub b: f64,
}

impl Scorer for TfIdf {
    fn score(
        &self,
        term_freq: u32,
        doc_length: u32,
        docs_containing_term: usize,
        num_docs: u32,
        _avg_dl: f64,
    ) -> f64 {
        let tf = term_freq as f64 / doc_length as f64;
        let idf = (num_docs as f64 / docs_containing_term as f64).ln();

        tf * idf
    }
}

impl Scorer for Bm25 {
    fn score(
        &self,
        term_freq: u32,
        doc_length: u32,
        docs_containing_term: usize,
        num_docs: u32,
        avg_dl: f64,
    ) -> f64 {
        let doc_length = doc_length as f64;
        let term_freq = term_freq as f64;
        let idf = (num_docs as f64 / docs_containing_term as f64).ln();

        idf * (term_freq * (self.k1 + 1.0))
            / (term_freq + self.k1 * (1.0 - self.b + self.b * doc_length / avg_dl))
    }
}

pub fn rank(
    candidates: Vec<DocId>,
    terms: &[String],
    index: &Index,
    scorer: &dyn Scorer,
) -> Vec<(DocId, f64)> {
    let mut results = Vec::new();
    for doc_id in candidates {
        let score: f64 = terms
            .iter()
            .filter_map(|term| {
                let postings = index.postings.get(term)?;
                let posting = postings.iter().find(|p| p.doc_id == doc_id)?;
                let doc_length = index.doc_lengths.get(&doc_id).copied().unwrap_or(1);
                let docs_containing_term = postings.len();
                Some(scorer.score(
                    posting.term_freq,
                    doc_length,
                    docs_containing_term,
                    index.num_docs,
                    index.avg_dl,
                ))
            })
            .sum();

        results.push((doc_id, score));
    }
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tfidf_higher_freq_scores_higher() {
        let scorer = TfIdf;
        let low = scorer.score(1, 100, 10, 1000, 0.0);
        let high = scorer.score(5, 100, 10, 1000, 0.0);
        assert!(high > low);
    }

    #[test]
    fn test_tfidf_rare_term_scores_higher() {
        let scorer = TfIdf;
        let common = scorer.score(3, 100, 500, 1000, 0.0);
        let rare = scorer.score(3, 100, 5, 1000, 0.0);
        assert!(rare > common);
    }

    #[test]
    fn test_bm25_higher_freq_scores_higher() {
        let scorer = Bm25 { k1: 1.2, b: 0.75 };
        let low = scorer.score(1, 100, 10, 1000, 100.0);
        let high = scorer.score(5, 100, 10, 1000, 100.0);
        assert!(high > low);
    }

    #[test]
    fn test_bm25_shorter_doc_scores_higher() {
        let scorer = Bm25 { k1: 1.2, b: 0.75 };
        let long_doc = scorer.score(3, 500, 10, 1000, 100.0);
        let short_doc = scorer.score(3, 50, 10, 1000, 100.0);
        assert!(short_doc > long_doc);
    }

    fn make_index(postings: Vec<(&str, Vec<(u32, u32)>)>, doc_lengths: Vec<(u32, u32)>) -> Index {
        let mut index = Index::new();
        for (term, docs) in postings {
            index.postings.insert(
                term.to_string(),
                docs.into_iter()
                    .map(|(doc_id, term_freq)| crate::index::postings::Posting {
                        doc_id,
                        term_freq,
                    })
                    .collect(),
            );
        }
        for (doc_id, length) in doc_lengths {
            index.doc_lengths.insert(doc_id, length);
        }
        index.num_docs = index.doc_lengths.len() as u32;
        index.avg_dl =
            index.doc_lengths.values().map(|&l| l as f64).sum::<f64>() / index.num_docs as f64;
        index
    }

    #[test]
    fn test_rank_orders_by_score_descending() {
        // 3 docs total, only 2 contain "rust"
        let index = make_index(
            vec![("rust", vec![(0, 1), (1, 5)])],
            vec![(0, 100), (1, 100), (2, 100)],
        );
        let scorer = Bm25 { k1: 1.2, b: 0.75 };
        let results = rank(vec![0, 1], &["rust".to_string()], &index, &scorer);

        assert_eq!(results[0].0, 1); // doc 1 has higher term_freq, should rank first
        assert!(results[0].1 > results[1].1);
    }

    #[test]
    fn test_rank_missing_term_skipped() {
        // 2 docs total, only 1 contains "rust"
        let index = make_index(vec![("rust", vec![(0, 3)])], vec![(0, 100), (1, 100)]);
        let scorer = Bm25 { k1: 1.2, b: 0.75 };
        let results = rank(
            vec![0],
            &["rust".to_string(), "nonexistent".to_string()],
            &index,
            &scorer,
        );

        assert_eq!(results.len(), 1);
        assert!(results[0].1 > 0.0);
    }
}
