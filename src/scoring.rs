use std::{collections::HashMap, sync::Arc};

use crate::{
    resources::LoadedResources,
    types::{ReferenceLabel, ReferenceRecord, ScoreRequest, ScoreResponse},
    vectorize::{vectorize, VectorizeError},
};

const K_NEIGHBORS: usize = 5;
const PARTITION_DIMS: [usize; 4] = [0, 3, 5, 12];
const PARTITION_WIDTHS: [f64; 4] = [0.05, 0.05, 0.05, 0.10];
const PARTITION_TARGET_CANDIDATES: usize = 256;
const PARTITION_MAX_RING: i16 = 1;

type BucketKey = [i16; PARTITION_DIMS.len()];

#[derive(Debug, Clone, Copy, PartialEq)]
struct TopKEntry {
    distance: f64,
    is_fraud: bool,
}

#[derive(Debug, Default)]
struct TopK<const K: usize> {
    entries: Vec<TopKEntry>,
}

impl<const K: usize> TopK<K> {
    fn new() -> Self {
        Self {
            entries: Vec::with_capacity(K),
        }
    }

    fn push(&mut self, distance: f64, is_fraud: bool) {
        let entry = TopKEntry { distance, is_fraud };

        if self.entries.len() < K {
            self.entries.push(entry);
            return;
        }

        let (worst_index, worst_entry) = self
            .entries
            .iter()
            .enumerate()
            .max_by(|left, right| left.1.distance.partial_cmp(&right.1.distance).unwrap())
            .unwrap();

        if distance < worst_entry.distance {
            self.entries[worst_index] = entry;
        }
    }

    fn into_sorted(mut self) -> Vec<TopKEntry> {
        self.entries
            .sort_by(|left, right| left.distance.partial_cmp(&right.distance).unwrap());
        self.entries
    }
}

#[derive(Debug, Default)]
struct PartitionIndex {
    buckets: HashMap<BucketKey, Vec<usize>>,
}

impl PartitionIndex {
    fn build(references: &[ReferenceRecord]) -> Self {
        let mut buckets: HashMap<BucketKey, Vec<usize>> = HashMap::new();

        for (index, reference) in references.iter().enumerate() {
            buckets
                .entry(bucket_key(&reference.vector))
                .or_default()
                .push(index);
        }

        Self { buckets }
    }

    fn candidate_indices(&self, vector: &[f64; 14], full_scan_count: usize) -> Vec<usize> {
        if self.buckets.is_empty() {
            return (0..full_scan_count).collect();
        }

        let center = bucket_key(vector);
        let mut current = center;
        let mut candidates = Vec::new();

        for ring in 0..=PARTITION_MAX_RING {
            collect_ring_candidates(&self.buckets, &center, ring, 0, &mut current, &mut candidates);

            if candidates.len() >= PARTITION_TARGET_CANDIDATES || candidates.len() >= K_NEIGHBORS {
                return candidates;
            }
        }

        (0..full_scan_count).collect()
    }
}

#[derive(Debug)]
pub struct ScoringEngine {
    normalization: crate::types::Normalization,
    mcc_risk: std::collections::HashMap<String, f64>,
    references: Vec<ReferenceRecord>,
    partition_index: PartitionIndex,
}

#[derive(Debug, thiserror::Error)]
pub enum ScoringError {
    #[error(transparent)]
    Vectorize(#[from] VectorizeError),
}

impl ScoringEngine {
    pub fn new(resources: LoadedResources) -> Self {
        let partition_index = PartitionIndex::build(&resources.references);

        Self {
            normalization: resources.normalization,
            mcc_risk: resources.mcc_risk,
            references: resources.references,
            partition_index,
        }
    }

    pub fn empty() -> Self {
        Self {
            normalization: crate::types::Normalization {
                max_amount: 10_000.0,
                max_installments: 12.0,
                amount_vs_avg_ratio: 10.0,
                max_minutes: 1_440.0,
                max_km: 1_000.0,
                max_tx_count_24h: 20.0,
                max_merchant_avg_amount: 10_000.0,
            },
            mcc_risk: std::collections::HashMap::new(),
            references: Vec::new(),
            partition_index: PartitionIndex::default(),
        }
    }

    pub fn score(&self, request: &ScoreRequest) -> Result<ScoreResponse, ScoringError> {
        if self.references.is_empty() {
            return Ok(ScoreResponse {
                approved: true,
                fraud_score: 0.0,
            });
        }

        let vector = vectorize(request, &self.normalization, &self.mcc_risk)?;
        let candidate_indices = self
            .partition_index
            .candidate_indices(&vector, self.references.len());
        let mut top_k = TopK::<K_NEIGHBORS>::new();

        for reference_index in candidate_indices {
            let reference = &self.references[reference_index];
            top_k.push(
                squared_distance(&vector, &reference.vector),
                reference.label == ReferenceLabel::Fraud,
            );
        }

        let nearest = top_k.into_sorted();
        let fraud_count = nearest.iter().filter(|entry| entry.is_fraud).count();
        let fraud_score = fraud_count as f64 / nearest.len() as f64;

        Ok(ScoreResponse {
            approved: fraud_score < 0.6,
            fraud_score,
        })
    }
}

pub type SharedScoringEngine = Arc<ScoringEngine>;

fn bucket_key(vector: &[f64; 14]) -> BucketKey {
    let mut key = [0_i16; PARTITION_DIMS.len()];

    for (slot, (&dimension, &width)) in PARTITION_DIMS.iter().zip(PARTITION_WIDTHS.iter()).enumerate() {
        let bucket = (vector[dimension] / width).floor();
        key[slot] = bucket.clamp(i16::MIN as f64, i16::MAX as f64) as i16;
    }

    key
}

fn collect_ring_candidates(
    buckets: &HashMap<BucketKey, Vec<usize>>,
    center: &BucketKey,
    ring: i16,
    dimension: usize,
    current: &mut BucketKey,
    candidates: &mut Vec<usize>,
) {
    if dimension == current.len() {
        let chebyshev_distance = current
            .iter()
            .zip(center.iter())
            .map(|(current_value, center_value)| (current_value - center_value).abs())
            .max()
            .unwrap_or(0);

        if chebyshev_distance == ring {
            if let Some(indices) = buckets.get(current) {
                candidates.extend(indices.iter().copied());
            }
        }
        return;
    }

    for offset in -ring..=ring {
        current[dimension] = center[dimension] + offset;
        collect_ring_candidates(buckets, center, ring, dimension + 1, current, candidates);
    }
}

fn squared_distance(left: &[f64; 14], right: &[f64; 14]) -> f64 {
    left.iter()
        .zip(right.iter())
        .map(|(left, right)| {
            let delta = left - right;
            delta * delta
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::{PartitionIndex, TopK};
    use crate::types::{ReferenceLabel, ReferenceRecord};

    #[test]
    fn fixed_top_k_keeps_smallest_distances_even_when_better_matches_arrive_late() {
        let mut top_k = TopK::<5>::new();

        for value in [10.0, 9.0, 8.0, 7.0, 6.0, 0.4, 0.3, 0.2, 0.1, 0.05] {
            top_k.push(value, value < 0.25);
        }

        let collected = top_k.into_sorted();
        let distances = collected
            .iter()
            .map(|entry| entry.distance)
            .collect::<Vec<_>>();
        let frauds = collected.iter().filter(|entry| entry.is_fraud).count();

        assert_eq!(distances, vec![0.05, 0.1, 0.2, 0.3, 0.4]);
        assert_eq!(frauds, 3);
    }

    #[test]
    fn partition_index_returns_local_candidates_without_scanning_distant_cluster() {
        let references = vec![
            ReferenceRecord {
                vector: [
                    0.10, 0.20, 0.30, 0.15, 0.33, 0.12, 0.01, 0.02, 0.10, 0.0, 1.0, 0.0, 0.10,
                    0.03,
                ],
                label: ReferenceLabel::Legit,
            },
            ReferenceRecord {
                vector: [
                    0.11, 0.20, 0.31, 0.16, 0.33, 0.11, 0.02, 0.03, 0.10, 0.0, 1.0, 0.0, 0.11,
                    0.03,
                ],
                label: ReferenceLabel::Fraud,
            },
            ReferenceRecord {
                vector: [
                    0.12, 0.19, 0.29, 0.14, 0.33, 0.13, 0.01, 0.02, 0.10, 0.0, 1.0, 0.0, 0.09,
                    0.03,
                ],
                label: ReferenceLabel::Legit,
            },
            ReferenceRecord {
                vector: [
                    0.13, 0.21, 0.30, 0.17, 0.33, 0.12, 0.03, 0.01, 0.10, 0.0, 1.0, 0.0, 0.10,
                    0.03,
                ],
                label: ReferenceLabel::Fraud,
            },
            ReferenceRecord {
                vector: [
                    0.14, 0.20, 0.32, 0.18, 0.33, 0.12, 0.01, 0.02, 0.10, 0.0, 1.0, 0.0, 0.12,
                    0.03,
                ],
                label: ReferenceLabel::Legit,
            },
            ReferenceRecord {
                vector: [
                    0.95, 0.80, 0.90, 0.90, 0.80, 0.95, 0.90, 0.90, 0.80, 1.0, 0.0, 1.0, 0.90,
                    0.90,
                ],
                label: ReferenceLabel::Fraud,
            },
            ReferenceRecord {
                vector: [
                    0.96, 0.81, 0.91, 0.91, 0.81, 0.96, 0.91, 0.91, 0.81, 1.0, 0.0, 1.0, 0.91,
                    0.91,
                ],
                label: ReferenceLabel::Fraud,
            },
        ];

        let partition_index = PartitionIndex::build(&references);
        let query = [
            0.115, 0.20, 0.30, 0.16, 0.33, 0.12, 0.02, 0.02, 0.10, 0.0, 1.0, 0.0, 0.10, 0.03,
        ];

        let candidate_indices = partition_index.candidate_indices(&query, references.len());

        assert_eq!(candidate_indices.len(), 5);
        assert!(candidate_indices.iter().all(|index| *index < 5));
    }
}
