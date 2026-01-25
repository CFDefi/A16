//! A16 Vector Index and Embedding Engine
//! 
//! Provides HNSW-based vector similarity search and TF-IDF text embedding.

mod hnsw;
mod embed;

#[cfg(test)]
mod tests;

pub use hnsw::{HNSWIndex, SearchResult};
pub use embed::{Embedder, TfIdfEmbedder};
