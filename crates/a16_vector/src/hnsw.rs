//! HNSW (Hierarchical Navigable Small World) Vector Index
//! 
//! A graph-based approximate nearest neighbor search algorithm.
//! Provides O(log n) insert and search with high recall.

use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;
use std::io::{self, Read, Write};
use std::fs::File;
use std::path::Path;

/// Result from a similarity search
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Index of the vector in the store
    pub id: usize,
    /// Cosine similarity score (higher = more similar)
    pub score: f32,
    /// Associated metadata key
    pub metadata_key: Option<String>,
}

impl PartialEq for SearchResult {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for SearchResult {}

impl PartialOrd for SearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchResult {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for min-heap behavior (we want max scores)
        other.score.partial_cmp(&self.score).unwrap_or(Ordering::Equal)
    }
}

/// HNSW Index for approximate nearest neighbor search
pub struct HNSWIndex {
    /// Stored vectors
    vectors: Vec<Vec<f32>>,
    /// Metadata keys for each vector
    metadata: Vec<Option<String>>,
    /// Graph edges per layer: layer -> node -> [(neighbor_id, distance)]
    layers: Vec<HashMap<usize, Vec<usize>>>,
    /// Entry point (highest layer node)
    entry_point: Option<usize>,
    /// Vector dimension
    dim: usize,
    /// Max connections per node
    m: usize,
    /// Max connections for layer 0
    m0: usize,
    /// Level multiplier (reserved for probabilistic layer assignment)
    #[allow(dead_code)]
    ml: f32,
    /// Search beam width
    ef_construction: usize,
}

impl std::fmt::Debug for HNSWIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HNSWIndex")
            .field("dim", &self.dim)
            .field("len", &self.vectors.len())
            .field("layers", &self.layers.len())
            .finish()
    }
}

impl HNSWIndex {
    /// Create a new HNSW index
    pub fn new(dim: usize) -> Self {
        Self {
            vectors: Vec::new(),
            metadata: Vec::new(),
            layers: vec![HashMap::new()],
            entry_point: None,
            dim,
            m: 16,
            m0: 32,
            ml: 1.0 / (16.0_f32).ln(),
            ef_construction: 100,
        }
    }
    
    /// Insert a vector with optional metadata key
    pub fn insert(&mut self, vector: Vec<f32>, metadata_key: Option<String>) -> usize {
        assert_eq!(vector.len(), self.dim, "Vector dimension mismatch");
        
        let id = self.vectors.len();
        self.vectors.push(vector);
        self.metadata.push(metadata_key);
        
        // Determine level for this node
        let level = self.random_level();
        
        // Ensure we have enough layers
        while self.layers.len() <= level {
            self.layers.push(HashMap::new());
        }
        
        // If first node, just set as entry point
        if self.entry_point.is_none() {
            self.entry_point = Some(id);
            for l in 0..=level {
                self.layers[l].insert(id, Vec::new());
            }
            return id;
        }
        
        let entry = self.entry_point.unwrap();
        let mut curr_entry = entry;
        let top_layer = self.layers.len() - 1;
        
        // Traverse from top to insertion level + 1
        for layer in (level + 1..=top_layer).rev() {
            curr_entry = self.search_layer_single(id, curr_entry, layer);
        }
        
        // Insert into layers from insertion level down to 0
        for layer in (0..=level.min(top_layer)).rev() {
            let neighbors = self.search_layer(id, curr_entry, self.ef_construction, layer);
            let m_max = if layer == 0 { self.m0 } else { self.m };
            
            // Select best neighbors
            let selected: Vec<usize> = neighbors.iter()
                .take(m_max)
                .map(|r| r.id)
                .collect();
            
            // Add bidirectional edges
            self.layers[layer].insert(id, selected.clone());
            for &neighbor in &selected {
                // Precompute similarities to avoid borrow conflict
                let edges = if let Some(e) = self.layers[layer].get(&neighbor) {
                    e.clone()
                } else {
                    continue;
                };
                
                // Compute similarities for all edges
                let mut edge_sims: Vec<(usize, f32)> = edges.iter()
                    .map(|&e| (e, self.cosine_similarity(neighbor, e)))
                    .collect();
                edge_sims.push((id, self.cosine_similarity(neighbor, id)));
                
                // Sort by similarity (highest first)
                edge_sims.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
                
                // Take top m_max
                let new_edges: Vec<usize> = edge_sims.iter()
                    .take(m_max)
                    .map(|(e, _)| *e)
                    .collect();
                
                self.layers[layer].insert(neighbor, new_edges);
            }
            
            if !neighbors.is_empty() {
                curr_entry = neighbors[0].id;
            }
        }
        
        // Update entry point if this node is in higher layer
        if level >= top_layer {
            self.entry_point = Some(id);
        }
        
        id
    }
    
    /// Search for k nearest neighbors
    pub fn search(&self, query: &[f32], k: usize, ef: usize) -> Vec<SearchResult> {
        if self.vectors.is_empty() {
            return Vec::new();
        }
        
        assert_eq!(query.len(), self.dim, "Query dimension mismatch");
        
        let entry = self.entry_point.unwrap();
        let mut curr_entry = entry;
        let top_layer = self.layers.len() - 1;
        
        // Traverse from top to layer 1
        for layer in (1..=top_layer).rev() {
            curr_entry = self.search_layer_single_query(query, curr_entry, layer);
        }
        
        // Search in layer 0
        let mut results = self.search_layer_query(query, curr_entry, ef.max(k), 0);
        results.truncate(k);
        
        // Add metadata keys
        for result in &mut results {
            result.metadata_key = self.metadata[result.id].clone();
        }
        
        results
    }
    
    /// Get dimension
    pub fn dim(&self) -> usize {
        self.dim
    }
    
    /// Get number of vectors
    pub fn len(&self) -> usize {
        self.vectors.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }
    
    /// Save index to file
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let mut file = File::create(path)?;
        
        // Header: magic, version, dim, num_vectors
        file.write_all(b"A16V")?;
        file.write_all(&1u32.to_le_bytes())?;
        file.write_all(&(self.dim as u32).to_le_bytes())?;
        file.write_all(&(self.vectors.len() as u32).to_le_bytes())?;
        
        // Vectors
        for vec in &self.vectors {
            for &v in vec {
                file.write_all(&v.to_le_bytes())?;
            }
        }
        
        // Metadata (simplified: just lengths and strings)
        for meta in &self.metadata {
            match meta {
                Some(s) => {
                    file.write_all(&(s.len() as u32).to_le_bytes())?;
                    file.write_all(s.as_bytes())?;
                }
                None => {
                    file.write_all(&0u32.to_le_bytes())?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Load index from file
    pub fn load(path: &Path) -> io::Result<Self> {
        let mut file = File::open(path)?;
        
        // Read header
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if &magic != b"A16V" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid magic"));
        }
        
        let mut buf4 = [0u8; 4];
        file.read_exact(&mut buf4)?;
        let _version = u32::from_le_bytes(buf4);
        
        file.read_exact(&mut buf4)?;
        let dim = u32::from_le_bytes(buf4) as usize;
        
        file.read_exact(&mut buf4)?;
        let num_vectors = u32::from_le_bytes(buf4) as usize;
        
        let mut index = Self::new(dim);
        
        // Read vectors
        for _ in 0..num_vectors {
            let mut vec = Vec::with_capacity(dim);
            for _ in 0..dim {
                file.read_exact(&mut buf4)?;
                vec.push(f32::from_le_bytes(buf4));
            }
            
            // Read metadata
            file.read_exact(&mut buf4)?;
            let meta_len = u32::from_le_bytes(buf4) as usize;
            let metadata = if meta_len > 0 {
                let mut meta_buf = vec![0u8; meta_len];
                file.read_exact(&mut meta_buf)?;
                Some(String::from_utf8_lossy(&meta_buf).to_string())
            } else {
                None
            };
            
            index.insert(vec, metadata);
        }
        
        Ok(index)
    }
    
    // === Private helpers ===
    
    fn random_level(&self) -> usize {
        let mut level = 0;
        let mut r: f32 = rand::random();
        while r < 0.5 && level < 16 {
            level += 1;
            r = rand::random();
        }
        level
    }
    
    fn cosine_similarity(&self, id1: usize, id2: usize) -> f32 {
        self.cosine_similarity_vec(&self.vectors[id1], &self.vectors[id2])
    }
    
    fn cosine_similarity_vec(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut dot = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;
        
        for i in 0..a.len() {
            dot += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }
        
        let denom = (norm_a * norm_b).sqrt();
        if denom > 1e-10 {
            dot / denom
        } else {
            0.0
        }
    }
    
    fn search_layer_single(&self, query_id: usize, entry: usize, layer: usize) -> usize {
        let query = &self.vectors[query_id];
        self.search_layer_single_query(query, entry, layer)
    }
    
    fn search_layer_single_query(&self, query: &[f32], entry: usize, layer: usize) -> usize {
        let mut curr = entry;
        let mut curr_sim = self.cosine_similarity_vec(query, &self.vectors[curr]);
        
        loop {
            let mut changed = false;
            
            if let Some(neighbors) = self.layers[layer].get(&curr) {
                for &neighbor in neighbors {
                    let sim = self.cosine_similarity_vec(query, &self.vectors[neighbor]);
                    if sim > curr_sim {
                        curr = neighbor;
                        curr_sim = sim;
                        changed = true;
                    }
                }
            }
            
            if !changed {
                break;
            }
        }
        
        curr
    }
    
    fn search_layer(&self, query_id: usize, entry: usize, ef: usize, layer: usize) -> Vec<SearchResult> {
        let query = &self.vectors[query_id];
        self.search_layer_query(query, entry, ef, layer)
    }
    
    fn search_layer_query(&self, query: &[f32], entry: usize, ef: usize, layer: usize) -> Vec<SearchResult> {
        let mut visited = HashSet::new();
        let mut candidates = BinaryHeap::new();
        let mut results = BinaryHeap::new();
        
        let entry_sim = self.cosine_similarity_vec(query, &self.vectors[entry]);
        candidates.push(SearchResult { id: entry, score: entry_sim, metadata_key: None });
        results.push(SearchResult { id: entry, score: -entry_sim, metadata_key: None }); // min-heap trick
        visited.insert(entry);
        
        while let Some(current) = candidates.pop() {
            // If worst result is better than best candidate, stop
            if let Some(worst) = results.peek() {
                if -worst.score > current.score {
                    break;
                }
            }
            
            if let Some(neighbors) = self.layers[layer].get(&current.id) {
                for &neighbor in neighbors {
                    if visited.insert(neighbor) {
                        let sim = self.cosine_similarity_vec(query, &self.vectors[neighbor]);
                        
                        // Add to candidates
                        candidates.push(SearchResult { id: neighbor, score: sim, metadata_key: None });
                        
                        // Add to results if better or not full
                        if results.len() < ef {
                            results.push(SearchResult { id: neighbor, score: -sim, metadata_key: None });
                        } else if let Some(worst) = results.peek() {
                            if sim > -worst.score {
                                results.pop();
                                results.push(SearchResult { id: neighbor, score: -sim, metadata_key: None });
                            }
                        }
                    }
                }
            }
        }
        
        // Convert to proper results (fix scores)
        let mut final_results: Vec<SearchResult> = results
            .into_iter()
            .map(|r| SearchResult { id: r.id, score: -r.score, metadata_key: None })
            .collect();
        
        final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
        final_results
    }
}
