use elasticsearch::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DistantError {
    #[error("Elasticsearch error: {0}")]
    ElasticsearchError(#[from] Error),

    #[error("General error: {0}")]
    GeneralError(String),
}