//! Integration tests for the embeddings module

use playrs::embedding::{
    example_embedding_with_model, get_all_embedding_models, get_embedding_model_by_name,
    list_embedding_model_names, similarity::CosineSimilarity, text_embedding,
};

#[test]
fn test_text_embedding_simple() {
    let doc = "This is a test document".to_string();
    let result = text_embedding(fastembed::EmbeddingModel::BGESmallENV15Q, doc);

    assert!(result.is_ok(), "Embedding generation should succeed");

    let embedding = result.unwrap();
    assert!(!embedding.is_empty(), "Embedding should not be empty");
    assert_eq!(
        embedding.len(),
        384,
        "AllMiniLML6V2 should produce 384-dimensional embeddings"
    );

    // Check that embeddings are valid floats
    for value in &embedding {
        assert!(value.is_finite(), "Embedding values should be finite");
    }
}

#[test]
fn test_text_embedding_empty_string() {
    let doc = "".to_string();
    let result = text_embedding(fastembed::EmbeddingModel::BGESmallENV15Q, doc);

    assert!(result.is_err(), "Empty string should return error");
    assert!(result.unwrap_err().to_string().contains("empty"));
}

#[test]
fn test_text_embedding_whitespace_only() {
    let doc = "   \n\t  ".to_string();
    let result = text_embedding(fastembed::EmbeddingModel::BGESmallENV15Q, doc);

    assert!(
        result.is_err(),
        "Whitespace-only string should return error"
    );
}

#[test]
fn test_text_embedding_different_lengths() {
    let short_doc = "Hi".to_string();
    let long_doc = "This is a much longer document with many more words and sentences. It should still produce a valid embedding of the same dimensionality as shorter documents.".to_string();

    let short_result = text_embedding(fastembed::EmbeddingModel::BGESmallENV15Q, short_doc);
    let long_result = text_embedding(fastembed::EmbeddingModel::BGESmallENV15Q, long_doc);

    assert!(short_result.is_ok());
    assert!(long_result.is_ok());

    let short_embedding = short_result.unwrap();
    let long_embedding = long_result.unwrap();

    assert_eq!(
        short_embedding.len(),
        long_embedding.len(),
        "Embeddings should have same dimensionality regardless of input length"
    );
}

#[test]
fn test_text_embedding_special_characters() {
    let doc = "Hello! @#$%^&*()_+ 你好 🚀 special chars".to_string();
    let result = text_embedding(fastembed::EmbeddingModel::BGESmallENV15Q, doc);

    assert!(result.is_ok(), "Text with special characters should work");

    let embedding = result.unwrap();
    assert_eq!(embedding.len(), 384);
}

#[test]
fn test_embedding_consistency() {
    let doc = "Consistent test document".to_string();

    let result1 = text_embedding(fastembed::EmbeddingModel::BGESmallENV15Q, doc.clone());
    let result2 = text_embedding(fastembed::EmbeddingModel::BGESmallENV15Q, doc.clone());

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    let embedding1 = result1.unwrap();
    let embedding2 = result2.unwrap();

    // Embeddings should be identical for the same input
    assert_eq!(embedding1.len(), embedding2.len());
    for (v1, v2) in embedding1.iter().zip(embedding2.iter()) {
        assert!((v1 - v2).abs() < 1e-6, "Embeddings should be consistent");
    }
}

#[test]
fn test_embedding_similarity() {
    let doc1 = "The cat sat on the mat".to_string();
    let doc2 = "A cat was sitting on a mat".to_string();
    let doc3 = "Quantum physics and relativity theory".to_string();

    let emb1 = text_embedding(fastembed::EmbeddingModel::AllMiniLML12V2, doc1).unwrap();
    let emb2 = text_embedding(fastembed::EmbeddingModel::AllMiniLML12V2, doc2).unwrap();
    let emb3 = text_embedding(fastembed::EmbeddingModel::AllMiniLML12V2, doc3).unwrap();

    // Compute cosine similarity
    let similarity_12 = &emb1.cosine_similarity(&emb2);
    let similarity_13 = &emb1.cosine_similarity(&emb3);

    // Similar documents should have higher similarity than dissimilar ones
    assert!(
        similarity_12 > similarity_13,
        "Similar documents should have higher cosine similarity"
    );
}

#[test]
fn test_text_embedding_signature() {
    // Test the exact function signature requested: fn text_embedding(doc: String) -> Result<Vec<f64>>
    let doc = "Test document for signature validation".to_string();
    let result: Result<Vec<f64>, Box<dyn std::error::Error>> =
        text_embedding(fastembed::EmbeddingModel::BGESmallENV15Q, doc);

    assert!(result.is_ok());
    let embedding = result.unwrap();
    assert_eq!(embedding.len(), 384);

    // Verify the return type is Vec<f64>
    for value in embedding {
        let _: f64 = value; // This will fail to compile if not f64
    }
}

#[test]
fn test_get_all_embedding_models() {
    let models = get_all_embedding_models();

    assert!(!models.is_empty(), "Should have embedding models available");

    // Test some known models
    assert!(models.contains_key("AllMiniLML6V2"));
    assert!(models.contains_key("BGEBaseENV15"));
    assert!(models.contains_key("NomicEmbedTextV1"));

    // Verify we can get the actual enum values
    let model = models.get("AllMiniLML6V2").unwrap();
    // This ensures the enum variant is correctly mapped
    let _ = format!("{:?}", model);
}

#[test]
fn test_get_embedding_model_by_name() {
    // Test valid model name
    let model = get_embedding_model_by_name("AllMiniLML6V2");
    assert!(model.is_some(), "Should find AllMiniLML6V2 model");

    // Test another valid model
    let model = get_embedding_model_by_name("BGEBaseENV15");
    assert!(model.is_some(), "Should find BGEBaseENV15 model");

    // Test invalid model name
    let model = get_embedding_model_by_name("NonExistentModel");
    assert!(model.is_none(), "Should not find non-existent model");

    // Test empty string
    let model = get_embedding_model_by_name("");
    assert!(model.is_none(), "Should not find model with empty name");
}

#[test]
fn test_list_embedding_model_names() {
    let names = list_embedding_model_names();

    assert!(!names.is_empty(), "Should have model names");

    // Test that names are sorted
    let mut sorted_names = names.clone();
    sorted_names.sort();
    assert_eq!(names, sorted_names, "Model names should be sorted");

    // Test some expected models are in the list
    assert!(names.contains(&"AllMiniLML6V2".to_string()));
    assert!(names.contains(&"BGEBaseENV15".to_string()));
    assert!(names.contains(&"NomicEmbedTextV1".to_string()));

    // Test uniqueness
    let mut unique_names = names.clone();
    unique_names.dedup();
    assert_eq!(
        names.len(),
        unique_names.len(),
        "All model names should be unique"
    );
}

#[test]
fn test_embedding_model_consistency() {
    // Test that all models returned by get_all_embedding_models
    // are also in the list_embedding_model_names
    let all_models = get_all_embedding_models();
    let name_list = list_embedding_model_names();

    for model_name in all_models.keys() {
        assert!(
            name_list.contains(model_name),
            "Model '{}' should be in the name list",
            model_name
        );
    }

    // Test that we can retrieve each model by name
    for name in &name_list {
        let model = get_embedding_model_by_name(name);
        assert!(
            model.is_some(),
            "Should be able to retrieve model '{}'",
            name
        );
    }
}

#[test]
fn test_example_embedding_with_model() {
    // Test with default model (None)
    let result = example_embedding_with_model(None, "Hello world");
    assert!(result.is_ok(), "Should succeed with default model");

    let embedding = result.unwrap();
    assert_eq!(
        embedding.len(),
        384,
        "Should produce 384-dimensional embedding"
    );

    // Test with specific model
    let result = example_embedding_with_model(Some("BGEBaseENV15"), "Hello world");
    assert!(result.is_ok(), "Should succeed with BGEBaseENV15 model");

    let embedding = result.unwrap();
    assert!(!embedding.is_empty(), "Should produce non-empty embedding");

    // Test with non-existent model (should fall back to default)
    let result = example_embedding_with_model(Some("NonExistentModel"), "Hello world");
    assert!(
        result.is_ok(),
        "Should succeed with fallback to default model"
    );
}
