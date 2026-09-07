use rhai::{Array, Dynamic, Map};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct VectorDoc {
    pub id: String,
    pub content: String,
    pub vector: Vec<f64>,
    pub score: f64,
}

#[derive(Clone)]
pub struct VectorEngine {
    collections: Arc<Mutex<HashMap<String, Vec<VectorDoc>>>>,
}

impl Default for VectorEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorEngine {
    pub fn new() -> Self {
        Self {
            collections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Compute lightweight character/word trigram TF-IDF embedding vector (32 dimensions)
    pub fn embed_text(text: &str) -> Vec<f64> {
        let mut vec = vec![0.0f64; 32];
        let lower = text.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();
        
        for (i, word) in words.iter().enumerate() {
            let hash = word.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
            let idx = (hash % 32) as usize;
            vec[idx] += 1.0 / ((i + 1) as f64).sqrt();
        }

        // Normalize vector
        let norm: f64 = vec.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 0.0 {
            for v in vec.iter_mut() {
                *v /= norm;
            }
        }
        vec
    }

    /// Insert or update a document in a vector collection
    pub fn upsert(&self, collection: &str, id: &str, content: &str) {
        let vector = Self::embed_text(content);
        let doc = VectorDoc {
            id: id.to_string(),
            content: content.to_string(),
            vector,
            score: 0.0,
        };

        if let Ok(mut cols) = self.collections.lock() {
            let list = cols.entry(collection.to_string()).or_default();
            if let Some(pos) = list.iter().position(|d| d.id == id) {
                list[pos] = doc;
            } else {
                list.push(doc);
            }
        }
    }

    /// Search a vector collection by semantic similarity
    pub fn search(&self, collection: &str, query: &str, top_k: usize) -> Result<Vec<VectorDoc>, String> {
        let query_vec = Self::embed_text(query);
        let cols = self.collections.lock().map_err(|e| e.to_string())?;

        let list = match cols.get(collection) {
            Some(l) => l,
            None => return Ok(Vec::new()),
        };

        let mut scored: Vec<VectorDoc> = list
            .iter()
            .map(|d| {
                let score = Self::cosine_similarity(&query_vec, &d.vector);
                VectorDoc {
                    id: d.id.clone(),
                    content: d.content.clone(),
                    vector: d.vector.clone(),
                    score,
                }
            })
            .collect();

        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        Ok(scored)
    }

    pub fn cosine_similarity(vec_a: &[f64], vec_b: &[f64]) -> f64 {
        if vec_a.is_empty() || vec_b.is_empty() || vec_a.len() != vec_b.len() {
            return 0.0;
        }

        let mut dot_product = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for (a, b) in vec_a.iter().zip(vec_b.iter()) {
            dot_product += a * b;
            norm_a += a * a;
            norm_b += b * b;
        }

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a.sqrt() * norm_b.sqrt())
    }

    pub fn to_f64_vec(arr: &Array) -> Vec<f64> {
        arr.iter()
            .filter_map(|d| {
                if let Ok(f) = d.as_float() {
                    Some(f)
                } else if let Ok(i) = d.as_int() {
                    Some(i as f64)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn rank_documents(query_vec: &[f64], docs: &[Map], vector_field: &str) -> Array {
        let mut scored: Vec<(f64, Map)> = docs
            .iter()
            .filter_map(|doc| {
                if let Some(dyn_vec) = doc.get(vector_field) {
                    if let Some(arr) = dyn_vec.clone().try_cast::<Array>() {
                        let doc_vec = Self::to_f64_vec(&arr);
                        let score = Self::cosine_similarity(query_vec, &doc_vec);
                        let mut copy = doc.clone();
                        copy.insert("similarity_score".into(), Dynamic::from(score));
                        return Some((score, copy));
                    }
                }
                None
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let mut result = Array::new();
        for (_, doc) in scored {
            result.push(Dynamic::from(doc));
        }
        result
    }
}
