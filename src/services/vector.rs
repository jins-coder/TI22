use rhai::{Array, Dynamic, Map};

pub struct VectorEngine;

impl VectorEngine {
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

        let mut results = Array::new();
        for (_, doc) in scored {
            results.push(Dynamic::from(doc));
        }
        results
    }
}
