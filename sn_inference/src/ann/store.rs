use crate::ann::{MinScoredIdx, Node, ScoredIdx};
use crate::error::{Error, Result};
use sn_core::types::message::Message;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TinyAnnStore {
    dim: usize,
    m: usize,               // max neighbors per node
    ef_construction: usize, // beam width for building
    ef_search: usize,       // beam width for queries

    // storage
    messages: Vec<Message>, // index -> message
    id_to_idx: HashMap<i32, usize>,

    // normalized embeddings (length 1.0)
    points: Vec<Vec<f32>>, // index -> normalized vector

    // graph
    graph: Vec<Node>,

    // optional: an entry point to start greedy search
    entry: Option<usize>,
}

impl TinyAnnStore {
    pub fn new(dim: usize, m: usize, ef_construction: usize, ef_search: usize) -> Self {
        Self {
            dim,
            m,
            ef_construction,
            ef_search,
            messages: Vec::new(),
            id_to_idx: HashMap::new(),
            points: Vec::new(),
            graph: Vec::new(),
            entry: None,
        }
    }

    #[inline]
    fn l2norm(v: &[f32]) -> f32 {
        v.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    #[inline]
    fn normalize(mut v: Vec<f32>) -> Vec<f32> {
        let n = Self::l2norm(&v);
        if n > 0.0 {
            for x in &mut v {
                *x /= n;
            }
        }
        v
    }

    #[inline]
    fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
        // with L2-normalized vectors, cosine == dot
        a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>()
    }

    /// Insert a message+embedding (embedding must have `self.dim` elements).
    pub fn insert(&mut self, message: Message) -> Result<()> {
        let embeddings = &message.embeddings;
        let id = message.id;

        if embeddings.len() != self.dim {
            return Err(Error::AnnInvalidEmbedding {
                id,
                expected_dim: self.dim,
                actual_dim: embeddings.len(),
            });
        }

        if self.id_to_idx.contains_key(&id) {
            return Err(Error::AnnDuplicateInsertId(id));
        }

        let idx = self.messages.len();
        let normalized = Self::normalize(embeddings.clone());

        self.messages.push(message);
        self.id_to_idx.insert(id, idx);
        self.points.push(normalized);
        self.graph.push(Node::default());

        if let Some(entry_idx) = self.entry {
            // connect new node using a greedy-then-beam search to pick neighbors
            let candidates =
                self.search_internal(&self.points[idx], self.ef_construction, Some(entry_idx));
            // pick top-M by score
            let mut picked: Vec<(f32, usize)> = candidates.into_iter().take(self.m).collect();

            // link both ways
            for &(_, nbr_idx) in &picked {
                self.graph[idx].neighbors.push(nbr_idx);
            }
            // ensure neighbor lists are capped to M with heuristic (keep highest score edges)
            for (_, nbr_idx) in picked.drain(..) {
                self.graph[nbr_idx].neighbors.push(idx);

                if self.graph[nbr_idx].neighbors.len() > self.m {
                    // prune: keep neighbors with best cosine to nbr
                    let nbr_vec = &self.points[nbr_idx];
                    self.graph[nbr_idx].neighbors.sort_unstable_by(|&a, &b| {
                        let sa = Self::cosine_sim(nbr_vec, &self.points[a]);
                        let sb = Self::cosine_sim(nbr_vec, &self.points[b]);
                        sb.partial_cmp(&sa).unwrap_or(Ordering::Equal)
                    });
                    self.graph[nbr_idx].neighbors.truncate(self.m);
                }
            }
        } else {
            // first element becomes entry
            self.entry = Some(idx);
        }

        Ok(())
    }

    /// Public KNN (top-k IDs + scores)
    pub fn knn(&self, query: &[f32], k: usize) -> Vec<(i32, f32)> {
        if self.messages.is_empty() {
            return vec![];
        }

        // normalize query for cosine
        let mut q = query.to_vec();
        let n = Self::l2norm(&q);
        if n > 0.0 {
            for x in &mut q {
                *x /= n;
            }
        }

        let start = self.entry.unwrap(); // safe: not empty
        let candidates = self.search_internal(&q, self.ef_search, Some(start));

        candidates
            .into_iter()
            .take(k)
            .map(|(score, idx)| (self.messages[idx].id, score))
            .collect()
    }

    /// Get back full messages (helper)
    pub fn get_messages(&self, ids: &[i32]) -> Vec<&Message> {
        ids.iter()
            .filter_map(|id| self.id_to_idx.get(id).map(|&i| &self.messages[i]))
            .collect()
    }

    /// Internal greedy+beam search returning a **max-heap sorted vector** of (score, idx)
    fn search_internal(&self, q: &[f32], ef: usize, start_idx: Option<usize>) -> Vec<(f32, usize)> {
        // best candidates we keep (min-heap to pop worst when exceeding ef)
        let mut best = BinaryHeap::<MinScoredIdx>::new();
        // frontier to explore (max-heap)
        let mut cand = BinaryHeap::<ScoredIdx>::new();
        let mut visited = vec![false; self.points.len()];

        let entry = start_idx.unwrap_or(0);
        let entry_score = Self::cosine_sim(q, &self.points[entry]);
        best.push(MinScoredIdx(ScoredIdx {
            score: entry_score,
            idx: entry,
        }));
        cand.push(ScoredIdx {
            score: entry_score,
            idx: entry,
        });
        visited[entry] = true;

        while let Some(ScoredIdx { score: _, idx }) = cand.pop() {
            // current worst in best set
            let mut worst_best = best.peek().map(|m| m.0.score).unwrap_or(f32::NEG_INFINITY);
            if best.len() == ef {
                worst_best = best.peek().unwrap().0.score;
            }

            // explore neighbors
            for &nb in &self.graph[idx].neighbors {
                if visited[nb] {
                    continue;
                }
                visited[nb] = true;

                let s = Self::cosine_sim(q, &self.points[nb]);
                // if promising vs worst in best set, consider it
                if best.len() < ef || s > worst_best {
                    cand.push(ScoredIdx { score: s, idx: nb });
                    best.push(MinScoredIdx(ScoredIdx { score: s, idx: nb }));
                    if best.len() > ef {
                        best.pop(); // drop current worst
                    }
                }
            }
        }

        // turn into sorted vec (desc)
        let mut out: Vec<(f32, usize)> = best.into_iter().map(|m| (m.0.score, m.0.idx)).collect();
        out.sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));
        out
    }
}
