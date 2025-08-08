//! Embedding module for text processing
//!
//! This module provides functionality for generating vector embeddings
//! from text using various embedding models.

pub mod command;
pub mod similarity;
pub mod text;

// Re-export the main functions for convenience
// Note: These are available but marked as unused until integrated into the main CLI
#[allow(unused_imports)]
pub use text::{
    example_embedding_with_model, get_all_embedding_models, get_embedding_model_by_name,
    list_embedding_model_names, text_embedding, EmbeddingError,
};
