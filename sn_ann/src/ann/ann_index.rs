use crate::ann::Region;
use crate::error::{ErrorAnn, Result};
use crate::utils::math::cosine_sim;
use sn_core::types::ann_item::AnnItem;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AnnIndex {
    dim: usize,
    centroids: Vec<Vec<f32>>,
    regions: HashMap<usize, Region>,
    last_checkpoint_info: HashMap<i32, i32>, // partition_id -> status
}

impl AnnIndex {
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            centroids: Vec::new(),
            last_checkpoint_info: HashMap::new(),
            regions: HashMap::new(),
        }
    }

    pub fn bulk_insert(&mut self, items: Vec<AnnItem>) -> Result<()> {
        for item in items {
            self.insert(item)?;
        }
        Ok(())
    }

    pub fn insert(&mut self, item: AnnItem) -> Result<()> {
        if (item.vectors.len() != self.dim) || item.vectors.is_empty() {
            return Err(ErrorAnn::DimMismatch {
                expected: self.dim,
                found: item.vectors.len(),
            });
        }

        // nearest centroid
        let (region_id, _) = self
            .centroids
            .iter()
            .enumerate()
            .map(|(i, c)| (i, cosine_sim(&item.vectors, c)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap_or((self.centroids.len(), 0.0));

        if region_id == self.centroids.len() {
            self.centroids.push(item.vectors.clone());
            self.regions.insert(region_id, Region::new());
        }

        self.regions.get_mut(&region_id).unwrap().insert(item);
        Ok(())
    }

    /// Performs k-nearest neighbor (k-NN) search for the given query vector.
    ///
    /// # Arguments
    ///
    /// * `q` - The query vector to search against.
    /// * `k` - The number of nearest neighbors to return.
    /// * `nprobe` - The number of nearest centroids (regions) to probe.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing references to the found `AnnItem`s and their similarity scores,
    /// sorted in descending order of similarity. The length of the returned vector is at most `k`.
    pub fn knn<'a>(&'a self, q: &[f32], k: usize, nprobe: usize) -> Vec<(&'a AnnItem, f32)> {
        if self.centroids.is_empty() {
            return vec![];
        }

        // pick nearest centroids
        let mut centroid_scores: Vec<(usize, f32)> = self
            .centroids
            .iter()
            .enumerate()
            .map(|(i, c)| (i, cosine_sim(q, c)))
            .collect();

        centroid_scores
            .sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        centroid_scores.truncate(nprobe);

        // search in selected regions
        let mut results: Vec<(&'a AnnItem, f32)> = Vec::new();
        for (region_id, _) in centroid_scores {
            if let Some(region) = self.regions.get(&region_id) {
                results.extend(region.knn(q, k));
            }
        }

        results.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);
        results
    }

    pub fn status(&self, partition: i32) -> i32 {
        match self.last_checkpoint_info.get(&partition) {
            Some(status) => *status,
            None => -1,
        }
    }
}
