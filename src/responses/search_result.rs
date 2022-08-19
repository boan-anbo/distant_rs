// Example code that deserializes and serializes the model.
// extern crate serde;
// #[macro_use]
// extern crate serde_derive;
// extern crate serde_json;
//
// use generated_module::[object Object];
//
// fn main() {
//     let json = r#"{"answer": 42}"#;
//     let model: [object Object] = serde_json::from_str(&json).unwrap();
// }

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DistantSearchResult {
    #[serde(rename = "_scroll_id")]
    pub scroll_id: String,

    #[serde(rename = "_shards")]
    pub shards: Shards,

    #[serde(rename = "hits")]
    pub hits: Hits,

    #[serde(rename = "timed_out")]
    pub timed_out: bool,

    #[serde(rename = "took")]
    pub took: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Hits {
    #[serde(rename = "hits")]
    pub hits: Vec<Hit>,

    #[serde(rename = "max_score")]
    pub max_score: f64,

    #[serde(rename = "total")]
    pub total: Total,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Hit {
    #[serde(rename = "_id")]
    pub id: String,

    #[serde(rename = "_index")]
    pub index: String,

    #[serde(rename = "_score")]
    pub score: f64,

    #[serde(rename = "_source")]
    pub source: Source,

    #[serde(rename = "_type")]
    pub hit_type: Type,

    #[serde(rename = "highlight")]
    pub highlight: Highlight,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Highlight {
    #[serde(rename = "text")]
    pub text: Vec<String>,

    #[serde(rename = "fileName")]
    pub file_name: Option<Vec<String>>,

    #[serde(rename = "filePath")]
    pub file_path: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
    #[serde(rename = "citeKey")]
    pub cite_key: String,

    #[serde(rename = "created")]
    pub created: Option<i64>,

    #[serde(rename = "fileName")]
    pub file_name: String,

    #[serde(rename = "filePath")]
    pub file_path: String,

    #[serde(rename = "modified")]
    pub modified: Option<i64>,

    #[serde(rename = "pageIndex")]
    pub page_index: i64,

    #[serde(rename = "pages")]
    pub pages: i64,

    #[serde(rename = "text")]
    pub text: String,

    #[serde(rename = "uuid")]
    pub uuid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Total {
    #[serde(rename = "relation")]
    pub relation: String,

    #[serde(rename = "value")]
    pub value: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Shards {
    #[serde(rename = "failed")]
    pub failed: i64,

    #[serde(rename = "skipped")]
    pub skipped: i64,

    #[serde(rename = "successful")]
    pub successful: i64,

    #[serde(rename = "total")]
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Type {
    #[serde(rename = "pdf")]
    Pdf,
}

