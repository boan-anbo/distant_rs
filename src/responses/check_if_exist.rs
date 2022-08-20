use serde::{Deserialize, Serialize};
use crate::responses::search_result::{Shards, Total};

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckIfFileExistsResult {
    #[serde(rename = "took")]
    pub took: i64,

    #[serde(rename = "timed_out")]
    pub timed_out: bool,

    #[serde(rename = "_shards")]
    pub shards: Shards,

    #[serde(rename = "hits")]
    pub hits: Hits,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Hits {
    #[serde(rename = "total")]
    pub total: Total,

    #[serde(rename = "max_score")]
    pub max_score: Option<serde_json::Value>,

    #[serde(rename = "hits")]
    pub hits: Vec<Option<serde_json::Value>>,
}
