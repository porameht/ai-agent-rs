use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding(pub Vec<f32>);

impl Embedding {
    pub fn new(vec: Vec<f32>) -> Self {
        Self(vec)
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.0
    }

    pub fn dimension(&self) -> usize {
        self.0.len()
    }

    pub fn into_inner(self) -> Vec<f32> {
        self.0
    }

    pub fn cosine_similarity(&self, other: &Embedding) -> f32 {
        if self.0.len() != other.0.len() || self.0.is_empty() {
            return 0.0;
        }

        let dot_product: f32 = self.0.iter().zip(other.0.iter()).map(|(a, b)| a * b).sum();
        let norm_a: f32 = self.0.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = other.0.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}

impl From<Vec<f32>> for Embedding {
    fn from(vec: Vec<f32>) -> Self {
        Self(vec)
    }
}

impl AsRef<[f32]> for Embedding {
    fn as_ref(&self) -> &[f32] {
        &self.0
    }
}
