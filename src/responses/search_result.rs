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

use carrel_commons::carrel::shared::search::v1::{CarrelSearchResponse, CarrelSearchResult, CarrelSearchResultHighlight, CarrelSearchResultItem, CarrelSearchResultMetadata};
use carrel_commons::generic::api::query::v1::SearchResultMetadata;
use serde::{Deserialize, Serialize};
use regex::Regex;

use lazy_static::lazy_static;

lazy_static! {
    static ref EM_REGEX: Regex = Regex::new(r"<em>(.*?)</em>").unwrap();
}
pub fn extract_em_content(input: &str) -> Vec<String> {
    EM_REGEX.captures_iter(input)
        .filter_map(|cap| cap.get(1))
        .map(|match_| match_.as_str().to_string())
        .collect()
}
impl From<DistantElasticSearchResult> for CarrelSearchResponse {
    fn from(result: DistantElasticSearchResult) -> Self {
        let mut carrel_search_results: Vec<CarrelSearchResult> = vec![];
        let mut carrel_search_results_metadata: SearchResultMetadata = SearchResultMetadata {
            result_total_items: result.hits.total.value as i32,
            ..Default::default()
        };
        let hits = result.hits.hits;
        for (index, hit) in hits.iter().enumerate() {
            let source = hit.source.clone();
            let carrel_search_result_item = source;
            let highlights: Vec<CarrelSearchResultHighlight> = hit.highlight.as_ref() // Convert to reference
                .map(|h| h.text.clone().unwrap_or_else(Vec::new)) // Work with reference
                .unwrap_or_else(Vec::new) // Provide default for None
                .iter()
                .map(|text| CarrelSearchResultHighlight {
                    field: "text".to_string(),
                    text: text.clone(),
                }).collect();
            let highlights_extracted: Vec<String> = highlights
                .iter()
                .flat_map(|highlight| extract_em_content(&highlight.text))
                .collect();
            let metadata: CarrelSearchResultMetadata = CarrelSearchResultMetadata {
                index: index as i32,
                score: hit.score as f32,
                highlights,
                highlights_extracted,
            };
            let carrel_search_result = CarrelSearchResult {
                result: Some(carrel_search_result_item),
                data: None,
                metadata: Some(metadata),
            };

            carrel_search_results.push(carrel_search_result);
        }

        CarrelSearchResponse {
            metadata: Some(carrel_search_results_metadata),
            results: carrel_search_results,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DistantElasticSearchResult {
    #[serde(rename = "_scroll_id")]
    pub scroll_id: Option<String>,

    #[serde(rename = "_shards")]
    pub shards: Option<Shards>,

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
    pub max_score: Option<f64>,

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
    pub source: CarrelSearchResultItem,

    #[serde(rename = "_type")]
    pub hit_type: Type,

    #[serde(rename = "highlight")]
    pub highlight: Option<Highlight>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Highlight {
    #[serde(rename = "text")]
    pub text: Option<Vec<String>>,

    #[serde(rename = "title")]
    pub title: Option<Vec<String>>,

    #[serde(rename = "filePath")]
    pub context: Option<Vec<String>>,

    #[serde(rename = "sourceName")]
    pub source_name: Option<Vec<String>>,

    #[serde(rename = "filePath")]
    pub file_path: Option<Vec<String>>,

    #[serde(rename = "uniqueId")]
    pub unique_id: Option<Vec<String>>,
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

#[cfg(test)]
mod test {
    #[test]
    fn test_extract_em_content() {
        let input = "<em>hello</em> <em>world</em>";
        let output = super::extract_em_content(input);
        assert_eq!(output, vec!["hello", "world"]);
    }
}

