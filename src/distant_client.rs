use std::borrow::Borrow;
use std::fs::File;
use std::io::{ErrorKind, Write};
use carrel_commons::carrel::shared::search::v1::CarrelSearchResultItem;
use carrel_commons::generic::api::query::v1::SearchQuery;
use elasticsearch::{BulkParts, DeleteParts, Elasticsearch, Error, IndexParts, ScrollParts, SearchParts};
use elasticsearch::cat::{CatIndices, CatIndicesParts};
use elasticsearch::http::request::JsonBody;
use elasticsearch::http::response::Response;
use elasticsearch::http::StatusCode;
use elasticsearch::http::transport::BuildError;
use elasticsearch::indices::IndicesDeleteParts;
use elasticsearch::params::Level::Indices;
use serde::Serialize;
use serde_json::{json, Value};
use log::info;
use crate::errors::DistantError;
use crate::responses::check_if_exist::CheckIfFileExistsResult;
use crate::responses::index_info::IndexInfo;
use crate::responses::search_result::{DistantElasticSearchResult};

pub struct DistantClient {
    user_name: String,
    password: String,
    endpoint: String,
    client: Elasticsearch,
    is_connected: bool,
}


impl DistantClient {
    pub fn new() -> Self {
        DistantClient {
            user_name: "".to_string(),
            password: "".to_string(),
            endpoint: String::new(),
            client: Elasticsearch::default(),
            is_connected: false,
        }
    }

    pub fn new_with_credentials(user_name: &str, password: &str, endpoint: &str) -> Self {
        DistantClient {
            user_name: user_name.to_string(),
            password: password.to_string(),
            endpoint: endpoint.to_string(),
            client: Elasticsearch::default(),
            is_connected: false,
        }
    }
}

pub struct ElasticInputEntry {
    pub data_type: String,
    pub item: CarrelSearchResultItem,
    pub unique_id: String,
}

// check health of the distant client
impl DistantClient {
    pub async fn index(&self, index_name: &str, entries: Vec<ElasticInputEntry>) -> Result<(), DistantError> {
        let mut bulk_body: Vec<JsonBody<_>> = Vec::new();

        for entry in entries {
            // Add the action metadata
            let action_metadata = json!({
                "index": {
                    "_type": entry.data_type,
                    "_index": index_name,
                    "_id": entry.unique_id,
                }
            });
            bulk_body.push(JsonBody::new(action_metadata));

            // Add the document body
            let document_body = json!(entry.item);
            bulk_body.push(JsonBody::new(document_body));
        }

        let response = self.client
            .bulk(BulkParts::Index(index_name))
            .body(bulk_body)
            .send().await;

        println!("{:?}", response);
        Ok(())
    }
    pub async fn check_health(&self) -> Result<String, Error> {
        let health = self.client.cat().health().send().await;
        match health {
            Ok(health) => {
                let response_body = health.text().await;
                response_body
            }
            Err(e) => {
                println!("{:?}", e);
                Err(e)
            }
        }
    }

    // search for documents in the elasticsearch index
    pub async fn search(&self,
                        index_name: String,
                        search_query: SearchQuery,
    ) -> Result<DistantElasticSearchResult, DistantError> {
        info!("Search query: {:?}", &search_query);
        let filter = search_query.filter;
        let mut query_text = String::new();
        let mut query_fields = Vec::new();
        match filter {
            Some(filter) => {
                query_text = filter.global_filter.unwrap_or("".to_string());
                query_fields = filter.global_filter_fields;
            }
            None => {}
        }

        let sort = search_query.sort;

        let sort_json = match sort {
            Some(sort) => {
                let sort_field = sort.field;
                let sort_order = sort.order;
                json!({ sort_field: { "order": sort_order } })
            }
            None => {
                json!({})
            }
        };

        let offset = search_query.offset;
        let length = search_query.length;

        let body_payload = json!({
                "size": length,
                "from": offset,
                "query": {
                    "multi_match": {
                        "query": query_text,
                        "fields": query_fields,
                        "fuzziness": "AUTO"
                    }
                },
                "sort": [sort_json],
                "highlight": {
                    "require_field_match": false,
                    "fields": {
                        "*": {
                            "pre_tags": ["<em>"],
                            "post_tags": ["</em>"]
                        }
                    }
                }
            });

        info!("Search query payload: {:?}", &body_payload);

        let reques_body_json = serde_json::to_string_pretty(&body_payload).unwrap();

// Assuming index_name is a String
        let index_parts = &[index_name.as_str()];

        let request_body = self.client
            .search(SearchParts::Index(index_parts))
            .from(offset as i64)
            .size(length as i64)
            // .scroll("1d")
            .body(body_payload);

        let result = request_body.send()
            .await?;
        // match status code
        match result.status_code() {
            StatusCode::OK => {
                let response_body: DistantElasticSearchResult = result.json::<DistantElasticSearchResult>().await?;
                Ok(response_body)
            }
            // catch and throw
            _ => {
                Err(DistantError::GeneralError("Error in search, check index name".to_string()))
            }
        }
    }

    pub async fn remove_index(&self, index_name: String) -> Result<(), DistantError> {
        let _result = self.client
            .indices()
            .delete(IndicesDeleteParts::Index(&[index_name.as_str()]))
            .send().await?;
        Ok(())
    }


    pub async fn remove_all_indices(&self) -> Result<(), DistantError> {
        let all_indices = self.list_indices().await?;
        for index in all_indices {
            let index_name = index.index;
            self.remove_index(index_name).await?;
        }
        Ok(())
    }

    // scroll
    pub async fn scroll(&self, scroll_id: &str) -> Result<DistantElasticSearchResult, Error> {
        let scroll = self.client.scroll(ScrollParts::ScrollId(scroll_id))
            .body(
                json! {
                    {
                        "scroll": "5m",
                    }
                }
            ).send().await;
        match scroll {
            Ok(scroll) => {
                let parsed_value = scroll.json::<DistantElasticSearchResult>().await.expect("Cannot unwrap scroll response");
                Ok(parsed_value)
            }
            Err(e) => {
                println!("{:?}", e);
                Err(e)
            }
        }
    }

    // list all indices in the elasticsearch
    pub async fn list_indices(&self) -> Result<Vec<IndexInfo>, Error> {
        let indices = self.client
            .cat()
            .indices(CatIndicesParts::None)
            .format("json")
            .send()
            .await;
        match indices {
            Ok(indices) => {
                let response_body = indices.json::<Vec<IndexInfo>>().await;
                response_body
            }
            Err(e) => {
                println!("{:?}", e);
                Err(e)
            }
        }
    }

    // check if file name exists in the elasticsearch
    pub async fn check_if_exist(&self, index: Vec<&str>, file_name: &str) -> Result<bool, Error> {
        let exists = self.search_by_filename(index, file_name, 0).await;
        match exists {
            Ok(result) => {
                // Ok(result.hits.total.value > 0)
                Ok(true)
            }
            Err(e) => {
                println!("{:?}", e);
                Err(e)
            }
        }
    }

    async fn search_by_filename(&self, index: Vec<&str>, file_name: &str, size: i64) -> Result<Value, Error> {
        let exists = self.client
            .search(SearchParts::Index(&index[..]))
            .scroll("1d")
            .size(size)
            .body(json! {
                {
                   "query":
                        {"term":
                            {
                              "fileName.keyword": file_name
                            }
                        },
                              "sort": [
            ],
                    }
            }).send().await;
        match exists {
            Ok(exists) => {
                let response_body = exists.json::<Value>().await;
                match response_body {
                    Ok(response_body) => {
                        Ok(response_body)
                    }
                    Err(e) => {
                        println!("{:?}", e);
                        Err(e)
                    }
                }
            }
            Err(e) => {
                println!("{:?}", e);
                Err(e)
            }
        }
    }
}

// tests
#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::Write;
    use carrel_commons::generic::api::query::v1::SearchFilter;
    use elasticsearch::cert::CertificateValidation::Default;
    use serde_json::to_string;
    use super::*;

    // test health
    #[tokio::test]
    async fn test_health() {
        let distant_client = DistantClient::new();
        let result = distant_client.check_health().await.unwrap();
        println!("{:?}", result);
    }


    #[tokio::test]
    async fn remove_all_indices() {
        let distant_client = DistantClient::new();
        let result = distant_client.remove_all_indices().await;

        // sleep for 2 seconds
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let all_indices = distant_client.list_indices().await.unwrap();
        assert_eq!(all_indices.len(), 0);
    }

    #[tokio::test]
    async fn test_index_function() -> Result<(), Box<dyn std::error::Error>> {
        // Create a DistantClient instance
        let client = DistantClient::new(/* ... configuration ... */);

        client.client.indices().delete(IndicesDeleteParts::Index(&["test_index"])).send().await?;


        // Define test data
        let test_entries = vec![
            ElasticInputEntry {
                data_type: "pdf".to_string(),
                item: CarrelSearchResultItem {
                    db_id: 0,
                    unique_id: "unique_a".to_string(),
                    unique_id_type: "citekey".to_string(),
                    material_type: 0,
                    title: "".to_string(),
                    text: "banana".to_string(),
                    context: "".to_string(),
                    source_type: 0,
                    source_id: None,
                    source_name: None,
                    file_path: None,
                    location: None,
                    location_type: None,
                    tags: vec![],
                },
                unique_id: "unique_a".to_string(),
            },
            ElasticInputEntry {
                data_type: "pdf".to_string(),
                item: CarrelSearchResultItem {
                    db_id: 0,
                    unique_id: "uniqueb".to_string(),
                    unique_id_type: "citekey".to_string(),
                    material_type: 0,
                    title: "".to_string(),
                    text: "apple".to_string(),
                    context: "".to_string(),
                    source_type: 0,
                    source_id: None,
                    source_name: None,
                    file_path: None,
                    location: None,
                    location_type: None,
                    tags: vec![]
                    ,
                },
                unique_id: "uniqueb".to_string(),
            },
        ];
        // Index data into a test index, e.g., "test_index"
        client.index("test_index", test_entries).await?;

        let listed_indices = client.list_indices().await?;
        println!("{:?}", listed_indices);
        assert_eq!(listed_indices.len(), 2);

        // wait for 5 seconds
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let search_result = client.search("test_index".to_string(), SearchQuery {
            relation: None,
            filter: Some(SearchFilter {
                must: vec![],
                any: vec![],
                global_filter: Some("appl".to_string()),
                global_filter_fields: vec!["text".to_string()],
            }),
            find_one: false,
            sort: None,
            offset: 0,
            length: 10,
            page: 0,
            find_all: false,
        }).await?;
        println!("{:?}", search_result);
        assert_eq!(search_result.hits.total.value, 1);

        // client.remove_index("test_index".to_string()).await?;

// List indices again
        let listed_indices = client.list_indices().await?;
        println!("{:?}", listed_indices);
        assert_eq!(listed_indices.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_list_indices_function() -> Result<(), Box<dyn std::error::Error>> {
        // Create a DistantClient instance
        let client = DistantClient::new(/* ... configuration ... */);

        // List all indices
        let indices = client.list_indices().await?;

        // Print the indices
        for index in indices {
            println!("Index: {:?}, docs: {:?}", index.index, index.docs_count);
        }

        Ok(())
    }

    // test list indices
    #[tokio::test]
    async fn test_list_indices() {
        let distant_client = DistantClient::new();
        let result = distant_client.list_indices().await.unwrap();

        // loop through the indices and print them
        for index in result {
            println!("Index: {:?}, docs: {:?}", index.index, index.docs_count);
        }
    }

    // test if file exists
    #[tokio::test]
    async fn test_check_if_exist() {
        let distant_client = DistantClient::new();
        let result = distant_client.check_if_exist(
            vec!["distant_rl_history"]
            , "test").await.unwrap();
        assert_eq!(result, false);
        let result_found = distant_client.check_if_exist(
            vec!["distant_rl_history"]
            , "[R] [[JTArtificial08]] Jones, TimM - 2008 - Artificial intelligence - a systems approach.pdf").await.unwrap();
        assert_eq!(result_found, true);
        let result_not_found = distant_client.check_if_exist(
            vec!["distant_rl_history"]
            , "[R] [[JTArtificial08]] Jones, TimM - 2008 - Artificial intelligence - a systems approach.pd").await.unwrap();
        assert_eq!(result_not_found, false);
    }

    // test search by filename TODO: fix the mismatch result struct between the search and search by term
    #[tokio::test]
    async fn test_search_by_filename() {
        let distant_client = DistantClient::new();
        let result = distant_client.search_by_filename(
            vec!["distant_rl_history"]
            , "[R] [[JTArtificial08]] Jones, TimM - 2008 - Artificial intelligence - a systems approach.pdf", 100).await.expect("no result");
        // write to file
        let mut file = File::create("test_search_by_filename.json").unwrap();
        file.write_all(result.to_string().as_bytes()).unwrap();
        // assert_eq!(result.hits.total.value, 0);
    }
}

