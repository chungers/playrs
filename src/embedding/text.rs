//! Text embedding module using FastEmbed
//!
//! This module provides functionality to compute vector embeddings for text input
//! using the FastEmbed library.

#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// Custom error type for embedding operations
#[derive(Debug)]
#[allow(dead_code)]
pub enum EmbeddingError {
    ModelInitialization(String),
    EmbeddingGeneration(String),
    InvalidInput(String),
}

impl fmt::Display for EmbeddingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmbeddingError::ModelInitialization(msg) => {
                write!(f, "Model initialization error: {}", msg)
            }
            EmbeddingError::EmbeddingGeneration(msg) => {
                write!(f, "Embedding generation error: {}", msg)
            }
            EmbeddingError::InvalidInput(msg) => {
                write!(f, "Invalid input error: {}", msg)
            }
        }
    }
}

impl Error for EmbeddingError {}

/// Compute vector embeddings for a given text document
///
/// # Arguments
/// * `doc` - The input text document to embed
///
/// # Returns
/// * `Result<Vec<f64>, Box<dyn Error>>` - Vector of embeddings or error
///
/// # Example
/// ```
/// use playrs::embedding::text_embedding;
/// use fastembed::EmbeddingModel;
///
/// let model = EmbeddingModel::BGESmallENV15Q;
/// let doc = "This is a sample text document".to_string();
/// let result = text_embedding(model, doc);
/// assert!(result.is_ok());
/// ```
#[allow(dead_code)]
pub fn text_embedding(model: EmbeddingModel, doc: String) -> Result<Vec<f64>, Box<dyn Error>> {
    trace!("Computing embeddings for document: {}", doc);

    // Validate input
    if doc.trim().is_empty() {
        return Err(Box::new(EmbeddingError::InvalidInput(
            "Document cannot be empty".to_string(),
        )));
    }

    // Initialize the embedding model
    let mut model =
        TextEmbedding::try_new(InitOptions::new(model).with_show_download_progress(false))
            .map_err(|e| {
                error!("Failed to initialize embedding model: {}", e);
                Box::new(EmbeddingError::ModelInitialization(e.to_string()))
            })?;

    // Generate embeddings
    let documents = vec![doc.as_str()];
    let embeddings = model.embed(documents, None).map_err(|e| {
        error!("Failed to generate embeddings: {}", e);
        Box::new(EmbeddingError::EmbeddingGeneration(e.to_string()))
    })?;

    // Extract the first (and only) embedding vector
    if let Some(embedding) = embeddings.into_iter().next() {
        let embedding_f64: Vec<f64> = embedding.into_iter().map(|x| x as f64).collect();
        trace!(
            "Generated embedding with {} dimensions",
            embedding_f64.len()
        );
        Ok(embedding_f64)
    } else {
        Err(Box::new(EmbeddingError::EmbeddingGeneration(
            "No embeddings generated".to_string(),
        )))
    }
}

/// Build a map of all the values in the EmbeddingModel enum
///
/// # Returns
/// * `HashMap<String, EmbeddingModel>` - Map of model names to EmbeddingModel variants
///
/// # Example
/// ```
/// use playrs::embedding::get_all_embedding_models;
///
/// let models = get_all_embedding_models();
/// assert!(models.contains_key("AllMiniLML6V2"));
/// ```
#[allow(dead_code)]
pub fn get_all_embedding_models() -> HashMap<String, EmbeddingModel> {
    let mut models = HashMap::new();

    // Only include confirmed variants that exist in the fastembed crate
    models.insert("AllMiniLML6V2".to_string(), EmbeddingModel::AllMiniLML6V2);
    models.insert("AllMiniLML12V2".to_string(), EmbeddingModel::AllMiniLML12V2);
    models.insert("BGEBaseENV15".to_string(), EmbeddingModel::BGEBaseENV15);
    models.insert("BGELargeENV15".to_string(), EmbeddingModel::BGELargeENV15);
    models.insert("BGESmallENV15".to_string(), EmbeddingModel::BGESmallENV15);
    models.insert("BGESmallZHV15".to_string(), EmbeddingModel::BGESmallZHV15);
    models.insert(
        "NomicEmbedTextV1".to_string(),
        EmbeddingModel::NomicEmbedTextV1,
    );
    models.insert(
        "NomicEmbedTextV15".to_string(),
        EmbeddingModel::NomicEmbedTextV15,
    );
    models.insert(
        "ParaphraseMLMiniLML12V2".to_string(),
        EmbeddingModel::ParaphraseMLMiniLML12V2,
    );
    models.insert(
        "MxbaiEmbedLargeV1".to_string(),
        EmbeddingModel::MxbaiEmbedLargeV1,
    );
    models.insert(
        "MultilingualE5Large".to_string(),
        EmbeddingModel::MultilingualE5Large,
    );
    models.insert(
        "MultilingualE5Base".to_string(),
        EmbeddingModel::MultilingualE5Base,
    );
    models.insert(
        "MultilingualE5Small".to_string(),
        EmbeddingModel::MultilingualE5Small,
    );

    models
}

/// Get embedding model by name
///
/// # Arguments
/// * `name` - The name of the embedding model
///
/// # Returns
/// * `Option<EmbeddingModel>` - The embedding model if found, None otherwise
///
/// # Example
/// ```
/// use playrs::embedding::get_embedding_model_by_name;
///
/// let model = get_embedding_model_by_name("AllMiniLML6V2");
/// assert!(model.is_some());
/// ```
#[allow(dead_code)]
pub fn get_embedding_model_by_name(name: &str) -> Option<EmbeddingModel> {
    get_all_embedding_models().get(name).cloned()
}

/// List all available embedding model names
///
/// # Returns
/// * `Vec<String>` - Vector of all available model names
///
/// # Example
/// ```
/// use playrs::embedding::list_embedding_model_names;
///
/// let names = list_embedding_model_names();
/// assert!(!names.is_empty());
/// ```
#[allow(dead_code)]
pub fn list_embedding_model_names() -> Vec<String> {
    let mut names: Vec<String> = get_all_embedding_models().keys().cloned().collect();
    names.sort();
    names
}

/// Example function demonstrating how to use the embedding model map
///
/// This function shows how to:
/// 1. Get all available models
/// 2. Look up a model by name
/// 3. Use the model for text embedding
///
/// # Arguments
/// * `model_name` - Optional model name to use (defaults to AllMiniLML6V2)
/// * `text` - Text to embed
///
/// # Returns
/// * `Result<Vec<f64>, Box<dyn Error>>` - The embedding vector
///
/// # Example
/// ```
/// use playrs::embedding::example_embedding_with_model;
///
/// let result = example_embedding_with_model(Some("BGEBaseENV15"), "Hello world");
/// assert!(result.is_ok());
/// ```
#[allow(dead_code)]
pub fn example_embedding_with_model(
    model_name: Option<&str>,
    text: &str,
) -> Result<Vec<f64>, Box<dyn Error>> {
    trace!("Example: Using embedding model map");

    // Build a map of all the values in the EmbeddingModel enum
    get_all_embedding_models(); // Make sure models are initialized
    trace!("Available models: {:?}", list_embedding_model_names());

    // Get the model to use
    let selected_model = match model_name {
        Some(name) => match get_embedding_model_by_name(name) {
            Some(model) => {
                trace!("Using requested model: {}", name);
                model
            }
            None => {
                warn!("Model '{}' not found, using default AllMiniLML6V2", name);
                EmbeddingModel::AllMiniLML6V2
            }
        },
        None => {
            trace!("Using default model: AllMiniLML6V2");
            EmbeddingModel::AllMiniLML6V2
        }
    };

    // Initialize the embedding model
    let mut model =
        TextEmbedding::try_new(InitOptions::new(selected_model).with_show_download_progress(false))
            .map_err(|e| {
                error!("Failed to initialize embedding model: {}", e);
                Box::new(EmbeddingError::ModelInitialization(e.to_string()))
            })?;

    // Generate embeddings
    let documents = vec![text];
    let embeddings = model.embed(documents, None).map_err(|e| {
        error!("Failed to generate embeddings: {}", e);
        Box::new(EmbeddingError::EmbeddingGeneration(e.to_string()))
    })?;

    // Extract the first (and only) embedding vector
    if let Some(embedding) = embeddings.into_iter().next() {
        let embedding_f64: Vec<f64> = embedding.into_iter().map(|x| x as f64).collect();
        trace!(
            "Generated embedding with {} dimensions",
            embedding_f64.len()
        );
        Ok(embedding_f64)
    } else {
        Err(Box::new(EmbeddingError::EmbeddingGeneration(
            "No embeddings generated".to_string(),
        )))
    }
}
