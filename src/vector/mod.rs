#![allow(dead_code)]

use std::collections::HashMap;

use crate::error::{Result, TywindbError};

/// Vector index for similarity search
pub struct VectorIndex {
    dimension: usize,
    vectors: HashMap<String, Vec<f64>>,
}

impl VectorIndex {
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            vectors: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: String, vector: Vec<f64>) -> Result<()> {
        if vector.len() != self.dimension {
            return Err(TywindbError::TypeMismatch {
                expected: format!("vector of dimension {}", self.dimension),
                actual: format!("vector of dimension {}", vector.len()),
            });
        }
        self.vectors.insert(id, vector);
        Ok(())
    }

    pub fn remove(&mut self, id: &str) -> bool {
        self.vectors.remove(id).is_some()
    }

    pub fn search(&self, query: &[f64], k: usize) -> Result<Vec<(String, f64)>> {
        if query.len() != self.dimension {
            return Err(TywindbError::TypeMismatch {
                expected: format!("vector of dimension {}", self.dimension),
                actual: format!("vector of dimension {}", query.len()),
            });
        }

        // Calculate distances to all vectors
        let mut distances: Vec<(String, f64)> = self
            .vectors
            .iter()
            .map(|(id, vec)| {
                let dist = cosine_distance(query, vec);
                (id.clone(), dist)
            })
            .collect();

        // Sort by distance (lower is more similar)
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return top k
        Ok(distances.into_iter().take(k).collect())
    }

    pub fn get_vector(&self, id: &str) -> Option<&Vec<f64>> {
        self.vectors.get(id)
    }

    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Calculate cosine distance between two vectors
pub fn cosine_distance(a: &[f64], b: &[f64]) -> f64 {
    let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0; // Maximum distance for zero vectors
    }

    // Cosine similarity
    let similarity = dot_product / (norm_a * norm_b);
    
    // Convert to distance (0 = identical, 2 = opposite)
    1.0 - similarity
}

/// Calculate Euclidean distance between two vectors
pub fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

/// Simple HNSW-like index (simplified for demonstration)
pub struct HnswIndex {
    dimension: usize,
    vectors: HashMap<String, Vec<f64>>,
    max_connections: usize,
    ef_construction: usize,
}

impl HnswIndex {
    pub fn new(dimension: usize, max_connections: usize, ef_construction: usize) -> Self {
        Self {
            dimension,
            vectors: HashMap::new(),
            max_connections,
            ef_construction,
        }
    }

    pub fn insert(&mut self, id: String, vector: Vec<f64>) -> Result<()> {
        if vector.len() != self.dimension {
            return Err(TywindbError::TypeMismatch {
                expected: format!("vector of dimension {}", self.dimension),
                actual: format!("vector of dimension {}", vector.len()),
            });
        }
        self.vectors.insert(id, vector);
        Ok(())
    }

    pub fn search(&self, query: &[f64], k: usize) -> Result<Vec<(String, f64)>> {
        if query.len() != self.dimension {
            return Err(TywindbError::TypeMismatch {
                expected: format!("vector of dimension {}", self.dimension),
                actual: format!("vector of dimension {}", query.len()),
            });
        }

        // For simplicity, we'll use brute-force search
        // In a real HNSW implementation, this would use the graph structure
        let mut distances: Vec<(String, f64)> = self
            .vectors
            .iter()
            .map(|(id, vec)| {
                let dist = cosine_distance(query, vec);
                (id.clone(), dist)
            })
            .collect();

        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(distances.into_iter().take(k).collect())
    }

    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_distance() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_distance(&a, &b) - 0.0).abs() < 1e-10);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_distance(&a, &c) - 1.0).abs() < 1e-10);

        let d = vec![-1.0, 0.0, 0.0];
        assert!((cosine_distance(&a, &d) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        assert!((euclidean_distance(&a, &b) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_vector_index() {
        let mut index = VectorIndex::new(3);
        
        index.insert("a".to_string(), vec![1.0, 0.0, 0.0]).unwrap();
        index.insert("b".to_string(), vec![0.0, 1.0, 0.0]).unwrap();
        index.insert("c".to_string(), vec![0.0, 0.0, 1.0]).unwrap();

        let results = index.search(&[1.0, 0.0, 0.0], 2).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "a");
        assert!(results[0].1 < 0.001);
    }
}
