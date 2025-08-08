#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use crate::embedding::{
    get_embedding_model_by_name, similarity::CosineSimilarity, text::get_all_embedding_models,
};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

use clap::{Args as clapArgs, Subcommand};

const DEFAULT_TEXT_EMBEDDING_MODEL: EmbeddingModel = EmbeddingModel::BGESmallENV15Q;

#[derive(Debug, clapArgs)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct Command {
    #[clap(subcommand)]
    pub modality: Modality,
}

#[derive(Debug, Subcommand)]
pub enum Modality {
    /// Text embedding operations
    Text {
        #[clap(subcommand)]
        verb: TextVerb,
    },
    /// Image embedding operations
    Image {
        #[clap(subcommand)]
        verb: ImageVerb,
    },
}

#[derive(Debug, Subcommand)]
pub enum TextVerb {
    /// List available text embedding models
    Models,
    /// Generate text embeddings
    Generate(TextGenerateArgs),
    /// Calculate text similarity
    Similarity(TextSimilarityArgs),
}

#[derive(Debug, Subcommand)]
pub enum ImageVerb {
    /// List available image embedding models
    Models,
    /// Generate image embeddings
    Generate(ImageGenerateArgs),
    /// Calculate image similarity
    Similarity(ImageSimilarityArgs),
}

#[derive(Debug, clapArgs)]
pub struct TextGenerateArgs {
    /// Model to use for embedding
    #[clap(short, long)]
    model: Option<String>,

    /// Text to compute the embedding
    doc: Vec<String>,
}

#[derive(Debug, clapArgs)]
pub struct TextSimilarityArgs {
    /// Model to use for embedding
    #[clap(short, long)]
    model: Option<String>,

    /// Metric to use for similarity computation
    #[clap(short = 'k', long)]
    metric: Option<String>,

    /// Text to compute the embedding
    doc: Vec<String>,
}

#[derive(Debug, clapArgs)]
pub struct ImageGenerateArgs {
    /// Model to use for embedding
    #[clap(short, long)]
    model: Option<String>,

    /// Path to the image
    path: Vec<String>,
}

#[derive(Debug, clapArgs)]
pub struct ImageSimilarityArgs {
    /// Model to use for embedding
    #[clap(short, long)]
    model: Option<String>,

    /// Metric to use for similarity computation
    #[clap(short = 'k', long)]
    metric: Option<String>,

    /// Text to compute the embedding
    doc: Vec<String>,
}

pub fn go(cmd: &Command) {
    trace!("Running command: {:?}", cmd);

    match &cmd.modality {
        Modality::Text { verb } => {
            trace!("Verb: {:?}", verb);
            match verb {
                TextVerb::Models => {
                    println!("Available models:");
                    let available_models = get_all_embedding_models();
                    for model in available_models.keys() {
                        println!("- {}", model);
                    }
                }
                TextVerb::Similarity(args) => {
                    print!("Computing similarity...{:?}", args);

                    if args.doc.len() < 2 {
                        println!("Error: At least two documents are required for similarity computation.");
                        return;
                    }

                    // First doc is the baseline to compare with other documents
                    let selected_model = match &args.model {
                        Some(model_name) => match get_embedding_model_by_name(model_name) {
                            Some(model) => &model.to_owned(),
                            None => &DEFAULT_TEXT_EMBEDDING_MODEL,
                        },
                        None => &DEFAULT_TEXT_EMBEDDING_MODEL,
                    };
                    trace!("Using model: {:?}", selected_model);
                    match TextEmbedding::try_new(
                        InitOptions::new(selected_model.clone()).with_show_download_progress(true),
                    ) {
                        Ok(mut model) => {
                            let docs = args.doc.clone();
                            match model.embed(docs, None) {
                                Ok(embeddings) => {
                                    print!("");
                                    // compare first with every other embedding
                                    let first_embedding = embeddings.first().unwrap();
                                    args.doc
                                        .iter()
                                        .skip(1)
                                        .zip(embeddings.iter().skip(1))
                                        .for_each(|(doc, embedding)| {
                                            let similarity =
                                                first_embedding.cosine_similarity(embedding);
                                            println!(
                                                "\nSimilarity: doc=\"{:?}\", cosine={:.2}%",
                                                doc,
                                                similarity * 100.0
                                            );
                                        });
                                }
                                Err(e) => {
                                    error!("Error embedding documents: {:?}", e);
                                }
                            }
                        }
                        Err(e) => error!("Error initializing model: {:?}", e),
                    }
                }
                TextVerb::Generate(args) => {
                    println!("Generating embeddings...");
                    let available_models = get_all_embedding_models();
                    let selected_model = match &args.model {
                        Some(model_name) => available_models
                            .get(model_name)
                            .unwrap_or(&DEFAULT_TEXT_EMBEDDING_MODEL),
                        None => &DEFAULT_TEXT_EMBEDDING_MODEL,
                    };
                    trace!("Using model: {:?}", selected_model);
                    match TextEmbedding::try_new(
                        InitOptions::new(selected_model.clone()).with_show_download_progress(true),
                    ) {
                        Ok(mut model) => {
                            let docs = args.doc.clone();
                            match model.embed(docs, None) {
                                Ok(embeddings) => {
                                    embeddings.iter().for_each(|embedding| {
                                        println!("\n\nEmbedding: {:?}", embedding);
                                    });
                                }
                                Err(e) => {
                                    error!("Error embedding documents: {:?}", e);
                                }
                            }
                        }
                        Err(e) => error!("Error initializing model: {:?}", e),
                    }
                }
            }
        }
        Modality::Image { verb } => {
            trace!("Command: {:?}", verb);
        }
    }
}
