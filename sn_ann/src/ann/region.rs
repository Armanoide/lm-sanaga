use sn_core::types::ann_item::AnnItem;

use crate::utils::math::cosine_sim;

#[derive(Debug, Clone)]
pub struct Region {
    points: Vec<Vec<f32>>, // vectors in this region
    pub items: Vec<AnnItem>,
}

impl Region {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            items: Vec::new(),
        }
    }

    pub fn insert(&mut self, item: AnnItem) {
        self.points.push(item.vectors.clone());
        self.items.push(item);
    }

    pub fn knn<'a>(&'a self, q: &[f32], k: usize) -> Vec<(&'a AnnItem, f32)> {
        let mut scored: Vec<(&AnnItem, f32)> = self
            .items
            .iter()
            .zip(&self.points)
            .map(|(item, p)| (item, cosine_sim(q, p)))
            .collect();

        scored.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored.truncate(k);
        scored
    }
}
