#![allow(dead_code)]

use std::collections::HashMap;

use crate::error::Result;

/// BM25 parameters
const K1: f64 = 1.5;
const B: f64 = 0.75;

/// Document for full-text search
#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub tokens: Vec<String>,
    pub term_frequencies: HashMap<String, usize>,
}

impl Document {
    pub fn new(id: String, content: String) -> Self {
        let tokens = tokenize(&content);
        let term_frequencies = compute_term_frequencies(&tokens);
        
        Self {
            id,
            content,
            tokens,
            term_frequencies,
        }
    }

    pub fn term_frequency(&self, term: &str) -> f64 {
        let tf = self.term_frequencies.get(term).copied().unwrap_or(0) as f64;
        tf / self.tokens.len() as f64
    }
}

/// Tokenize text into words
pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Compute term frequencies
fn compute_term_frequencies(tokens: &[String]) -> HashMap<String, usize> {
    let mut freq = HashMap::new();
    for token in tokens {
        *freq.entry(token.clone()).or_insert(0) += 1;
    }
    freq
}

/// BM25 Search Index
pub struct SearchIndex {
    documents: HashMap<String, Document>,
    document_count: usize,
    average_document_length: f64,
    inverted_index: HashMap<String, Vec<String>>, // term -> document ids
}

impl Default for SearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchIndex {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            document_count: 0,
            average_document_length: 0.0,
            inverted_index: HashMap::new(),
        }
    }

    pub fn add_document(&mut self, id: String, content: String) {
        let doc = Document::new(id.clone(), content);
        
        // Update inverted index
        for token in &doc.tokens {
            self.inverted_index
                .entry(token.clone())
                .or_default()
                .push(id.clone());
        }
        
        // Update statistics
        self.document_count += 1;
        self.average_document_length = 
            (self.average_document_length * (self.document_count - 1) as f64 + doc.tokens.len() as f64) 
            / self.document_count as f64;
        
        self.documents.insert(id, doc);
    }

    pub fn remove_document(&mut self, id: &str) -> bool {
        if let Some(doc) = self.documents.remove(id) {
            // Update inverted index
            for token in &doc.tokens {
                if let Some(ids) = self.inverted_index.get_mut(token) {
                    ids.retain(|x| x != id);
                    if ids.is_empty() {
                        self.inverted_index.remove(token);
                    }
                }
            }
            
            // Update statistics
            self.document_count -= 1;
            if self.document_count > 0 {
                self.average_document_length = 
                    (self.average_document_length * (self.document_count + 1) as f64 - doc.tokens.len() as f64) 
                    / self.document_count as f64;
            } else {
                self.average_document_length = 0.0;
            }
            
            true
        } else {
            false
        }
    }

    pub fn search(&self, query: &str, k: usize) -> Result<Vec<(String, f64, String)>> {
        let query_tokens = tokenize(query);
        
        if query_tokens.is_empty() {
            return Ok(Vec::new());
        }

        // Calculate BM25 scores for each document
        let mut scores: HashMap<String, f64> = HashMap::new();

        for query_token in &query_tokens {
            // Get documents containing this term
            if let Some(doc_ids) = self.inverted_index.get(query_token) {
                // IDF (Inverse Document Frequency)
                let df = doc_ids.len() as f64;
                let idf = ((self.document_count as f64 - df + 0.5) / (df + 0.5) + 1.0).ln();

                for doc_id in doc_ids {
                    if let Some(doc) = self.documents.get(doc_id) {
                        // TF (Term Frequency) with BM25 normalization
                        let tf = doc.term_frequencies.get(query_token).copied().unwrap_or(0) as f64;
                        let doc_len = doc.tokens.len() as f64;
                        
                        let tf_normalized = (tf * (K1 + 1.0)) / 
                            (tf + K1 * (1.0 - B + B * doc_len / self.average_document_length));
                        
                        let score = idf * tf_normalized;
                        
                        *scores.entry(doc_id.clone()).or_insert(0.0) += score;
                    }
                }
            }
        }

        // Sort by score
        let mut results: Vec<(String, f64, String)> = scores
            .into_iter()
            .filter_map(|(id, score)| {
                self.documents.get(&id).map(|doc| (id, score, doc.content.clone()))
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results.into_iter().take(k).collect())
    }

    pub fn get_document(&self, id: &str) -> Option<&Document> {
        self.documents.get(id)
    }

    pub fn len(&self) -> usize {
        self.documents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let tokens = tokenize("Hello, World! This is a test.");
        assert_eq!(tokens, vec!["hello", "world", "this", "is", "a", "test"]);
    }

    #[test]
    fn test_bm25_search() {
        let mut index = SearchIndex::new();
        
        index.add_document(
            "doc1".to_string(),
            "The quick brown fox jumps over the lazy dog".to_string(),
        );
        index.add_document(
            "doc2".to_string(),
            "The lazy dog sleeps all day".to_string(),
        );
        index.add_document(
            "doc3".to_string(),
            "The quick brown fox is very fast".to_string(),
        );

        let results = index.search("quick fox", 2).unwrap();
        assert_eq!(results.len(), 2);
        // doc3 is shorter, so BM25 ranks it higher due to length normalization
        assert_eq!(results[0].0, "doc3");
        assert_eq!(results[1].0, "doc1");
    }
}
