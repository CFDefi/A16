//! Tests for a16_vector

use crate::hnsw::HNSWIndex;
use crate::embed::{Embedder, TfIdfEmbedder, BagOfWordsEmbedder};

#[test]
fn test_hnsw_insert_search() {
    let mut index = HNSWIndex::new(4);
    
    // Insert some vectors
    index.insert(vec![1.0, 0.0, 0.0, 0.0], Some("a".to_string()));
    index.insert(vec![0.0, 1.0, 0.0, 0.0], Some("b".to_string()));
    index.insert(vec![0.0, 0.0, 1.0, 0.0], Some("c".to_string()));
    index.insert(vec![0.9, 0.1, 0.0, 0.0], Some("d".to_string()));
    
    assert_eq!(index.len(), 4);
    
    // Search for similar to [1, 0, 0, 0]
    let results = index.search(&[1.0, 0.0, 0.0, 0.0], 2, 10);
    
    assert!(!results.is_empty());
    // First result should be "a" (exact match) or "d" (very similar)
    assert!(results[0].metadata_key == Some("a".to_string()) || 
            results[0].metadata_key == Some("d".to_string()));
}

#[test]
fn test_hnsw_cosine_similarity() {
    let index = HNSWIndex::new(3);
    
    // Test that similar vectors get high scores
    let mut idx = HNSWIndex::new(3);
    idx.insert(vec![1.0, 0.0, 0.0], None);
    idx.insert(vec![0.9, 0.1, 0.0], None);
    
    let results = idx.search(&[1.0, 0.0, 0.0], 2, 10);
    assert!(results[0].score > 0.99); // Near-exact match
}

#[test]
fn test_hnsw_empty() {
    let index = HNSWIndex::new(4);
    let results = index.search(&[1.0, 0.0, 0.0, 0.0], 5, 10);
    assert!(results.is_empty());
}

#[test]
fn test_tfidf_embedder() {
    let mut embedder = TfIdfEmbedder::new(32);
    
    // Fit on corpus
    embedder.fit(&[
        "hello world",
        "goodbye world", 
        "hello there",
    ]);
    
    let vec1 = embedder.embed("hello world");
    let vec2 = embedder.embed("hello world");
    let vec3 = embedder.embed("goodbye there");
    
    assert_eq!(vec1.len(), 32);
    
    // Same input should give same output
    for i in 0..32 {
        assert!((vec1[i] - vec2[i]).abs() < 1e-6);
    }
    
    // Vector should be normalized
    let norm: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 0.01 || norm < 0.01); // Either normalized or zero
}

#[test]
fn test_bow_embedder() {
    let embedder = BagOfWordsEmbedder::new(16);
    
    let vec1 = embedder.embed("hello world");
    let vec2 = embedder.embed("hello");
    
    assert_eq!(vec1.len(), 16);
    assert_eq!(vec2.len(), 16);
    
    // Should be normalized
    let norm: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 0.01);
}

#[test]
fn test_empty_text() {
    let embedder = BagOfWordsEmbedder::new(8);
    let vec = embedder.embed("");
    assert_eq!(vec.len(), 8);
    // Should be all zeros
    assert!(vec.iter().all(|&x| x == 0.0));
}
