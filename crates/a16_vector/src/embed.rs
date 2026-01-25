//! Text Embedding Engine
//! 
//! Provides TF-IDF based text embedding for offline vector generation.

use std::collections::HashMap;

/// Trait for embedding text to vectors
pub trait Embedder {
    fn embed(&self, text: &str) -> Vec<f32>;
    fn dim(&self) -> usize;
}

/// TF-IDF based embedder using feature hashing
pub struct TfIdfEmbedder {
    dim: usize,
    idf: HashMap<String, f32>,
    doc_count: usize,
}

impl TfIdfEmbedder {
    /// Create new embedder with specified dimension
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            idf: HashMap::new(),
            doc_count: 0,
        }
    }
    
    /// Fit IDF values from corpus
    pub fn fit(&mut self, documents: &[&str]) {
        self.doc_count = documents.len();
        let mut doc_freq: HashMap<String, usize> = HashMap::new();
        
        for doc in documents {
            let tokens: Vec<String> = self.tokenize(doc);
            let unique: std::collections::HashSet<_> = tokens.into_iter().collect();
            for token in unique {
                *doc_freq.entry(token).or_insert(0) += 1;
            }
        }
        
        // Calculate IDF: log(N / df)
        let n = self.doc_count as f32;
        for (token, df) in doc_freq {
            let idf = (n / (df as f32 + 1.0)).ln() + 1.0;
            self.idf.insert(token, idf);
        }
    }
    
    /// Tokenize text into words
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty() && s.len() > 1)
            .map(|s| s.to_string())
            .collect()
    }
    
    /// Hash a token to a bucket
    fn hash_token(&self, token: &str) -> usize {
        let mut hash: u64 = 5381;
        for byte in token.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        (hash as usize) % self.dim
    }
}

impl Embedder for TfIdfEmbedder {
    fn embed(&self, text: &str) -> Vec<f32> {
        let mut vec = vec![0.0f32; self.dim];
        let tokens = self.tokenize(text);
        
        if tokens.is_empty() {
            return vec;
        }
        
        // Count term frequencies
        let mut tf: HashMap<String, usize> = HashMap::new();
        for token in &tokens {
            *tf.entry(token.clone()).or_insert(0) += 1;
        }
        
        // Calculate TF-IDF and hash to buckets
        let total = tokens.len() as f32;
        for (token, count) in tf {
            let tf_val = (count as f32) / total;
            let idf_val = self.idf.get(&token).copied().unwrap_or(1.0);
            let tfidf = tf_val * idf_val;
            
            let bucket = self.hash_token(&token);
            vec[bucket] += tfidf;
        }
        
        // L2 normalize
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-10 {
            for v in &mut vec {
                *v /= norm;
            }
        }
        
        vec
    }
    
    fn dim(&self) -> usize {
        self.dim
    }
}

/// Simple bag-of-words embedder (no IDF, just hashed counts)
#[allow(dead_code)]
pub struct BagOfWordsEmbedder {
    dim: usize,
}

#[allow(dead_code)]
impl BagOfWordsEmbedder {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }
    
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty() && s.len() > 1)
            .map(|s| s.to_string())
            .collect()
    }
    
    fn hash_token(&self, token: &str) -> usize {
        let mut hash: u64 = 5381;
        for byte in token.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        (hash as usize) % self.dim
    }
}

impl Embedder for BagOfWordsEmbedder {
    fn embed(&self, text: &str) -> Vec<f32> {
        let mut vec = vec![0.0f32; self.dim];
        let tokens = self.tokenize(text);
        
        for token in tokens {
            let bucket = self.hash_token(&token);
            vec[bucket] += 1.0;
        }
        
        // L2 normalize
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-10 {
            for v in &mut vec {
                *v /= norm;
            }
        }
        
        vec
    }
    
    fn dim(&self) -> usize {
        self.dim
    }
}
