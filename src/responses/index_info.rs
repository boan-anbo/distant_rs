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

use serde::{Serialize, Deserialize};

pub type Index = Vec<IndexInfo>;

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexInfo {
    #[serde(rename = "health")]
    pub health: String,

    #[serde(rename = "status")]
    pub status: String,

    #[serde(rename = "index")]
    pub index: String,

    #[serde(rename = "uuid")]
    pub uuid: String,

    #[serde(rename = "pri")]
    pub pri: String,

    #[serde(rename = "rep")]
    pub rep: String,

    #[serde(rename = "docs.count")]
    pub docs_count: String,

    #[serde(rename = "docs.deleted")]
    pub docs_deleted: String,

    #[serde(rename = "store.size")]
    pub store_size: String,

    #[serde(rename = "pri.store.size")]
    pub pri_store_size: String,
}
