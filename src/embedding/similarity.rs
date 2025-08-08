pub trait CosineSimilarity {
    fn cosine_similarity(self, other: Self) -> f64;
}

// Specialized implementation for f32 vectors with optimized precision
impl CosineSimilarity for &[f32] {
    fn cosine_similarity(self, other: Self) -> f64 {
        let dot_product = self
            .iter()
            .zip(other.iter())
            .map(|(x, y)| (*x as f64) * (*y as f64))
            .sum::<f64>();

        let norm_a = self.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();

        let norm_b = other
            .iter()
            .map(|x| (*x as f64).powi(2))
            .sum::<f64>()
            .sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
}

// Implementation for Vec<f32> references
impl CosineSimilarity for &Vec<f32> {
    fn cosine_similarity(self, other: Self) -> f64 {
        self.as_slice().cosine_similarity(other.as_slice())
    }
}

// Specialized implementation for f64 vectors with optimized precision
impl CosineSimilarity for &[f64] {
    fn cosine_similarity(self, other: Self) -> f64 {
        let dot_product = self
            .iter()
            .zip(other.iter())
            .map(|(x, y)| (*x as f64) * (*y as f64))
            .sum::<f64>();

        let norm_a = self.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();

        let norm_b = other
            .iter()
            .map(|x| (*x as f64).powi(2))
            .sum::<f64>()
            .sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
}

// Implementation for Vec<f32> references
impl CosineSimilarity for &Vec<f64> {
    fn cosine_similarity(self, other: Self) -> f64 {
        self.as_slice().cosine_similarity(other.as_slice())
    }
}

pub trait DotProductSimilarity {
    fn dotproduct_similarity(self, other: Self) -> f64;
}

// Specialized implementation for f32 vectors with optimized precision
impl DotProductSimilarity for &[f32] {
    fn dotproduct_similarity(self, other: Self) -> f64 {
        self.iter()
            .zip(other.iter())
            .map(|(x, y)| (*x as f64) * (*y as f64))
            .sum::<f64>()
    }
}

// Implementation for Vec<f32> references
impl DotProductSimilarity for &Vec<f32> {
    fn dotproduct_similarity(self, other: Self) -> f64 {
        self.as_slice().dotproduct_similarity(other.as_slice())
    }
}
