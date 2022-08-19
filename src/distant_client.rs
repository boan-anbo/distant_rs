use std::borrow::Borrow;
use std::io::ErrorKind;
use elasticsearch::{Elasticsearch, Error, ScrollParts, SearchParts};
use elasticsearch::cat::{CatIndices, CatIndicesParts};
use elasticsearch::http::StatusCode;
use elasticsearch::http::transport::BuildError;
use serde_json::{json, Value};
use crate::responses::index_info::IndexInfo;
use crate::responses::search_result::{DistantSearchResult};

pub struct DistantClient {
    endpoint: String,
    client: Elasticsearch,
    is_connected: bool,
}


impl DistantClient {
    pub async fn new() -> Self {
        DistantClient {
            endpoint: String::new(),
            client: Elasticsearch::default(),
            is_connected: false,
        }
    }
}

// check health of the distant client
impl DistantClient {
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
    pub async fn search(&self, index: Vec<&str>, query: &str) -> Result<DistantSearchResult, Error> {
        let search = self.client
            .search(SearchParts::Index(&index[..]))
            .size(100)
            .scroll("1d")
            .body(json! {
                 {
            "query": {
                "query_string": {
                    "query": query,
                    "fields": ["text^5", "fileName"],
                    "tie_breaker": 0
                }
            },
            "sort": [
                { "_score": { "order": "desc" } },
                // {"created": {"order": "asc"}}
            ],
            "highlight": {
                "require_field_match": false,
                "fields": {
                    '*': {
                        "pre_tags": ["<em>"],
                        "post_tags": ["</em>"]
                    }
                }
                    }
                }
            })
            .send().await;
        match search {
            Ok(search) => {
                println!("{:?}", search);

                // match status code
                match search.status_code() {
                    StatusCode::OK => {
                        let response_body = search.json::<DistantSearchResult>().await;
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
                    // catch and throw
                    _ => {
                        Err(Error::from(BuildError::from(std::io::Error::new(ErrorKind::Other, "search error; check index name"))))
                    }
                }
            }
            Err(e) => {
                println!("{:?}", e);
                Err(e)
            }
        }

    }

    // scroll
    pub async fn scroll(&self, scroll_id: &str) -> Result<DistantSearchResult, Error> {
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
                let parsed_value = scroll.json::<DistantSearchResult>().await.expect("Cannot unwrap scroll response");
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
}

// tests
#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::Write;
    use super::*;

    // test health
    #[tokio::test]
    async fn test_health() {
        let distant_client = DistantClient::new().await;
        let result = distant_client.check_health().await.unwrap();
        println!("{:?}", result);
    }

    // test search
    #[tokio::test]
    async fn test_search() {
        let distant_client = DistantClient::new().await;
        let result = distant_client.search(
            vec!["distant_rl_history"]
            , "test").await;

        // assert result is not null
        assert!(result.is_ok());
        assert!(result.unwrap().scroll_id.len() > 0);

    }

    // test scroll
    #[tokio::test]
    async fn test_scroll() {
        let distant_client = DistantClient::new().await;
        let result = distant_client.search(
            vec!["distant_rl_history"]
            , "test").await;
        let scroll_id_from_first = &result.as_ref().unwrap().scroll_id;
        let scroll_result = distant_client.scroll(&scroll_id_from_first).await;
        // write to file

        // assert same scroll id
        assert!(scroll_result.is_ok());
        assert_eq!(scroll_id_from_first.as_ref(), scroll_result.as_ref().unwrap().scroll_id);
        // assert different first result
        assert_ne!(result.unwrap().hits.hits[0].id, scroll_result.as_ref().unwrap().hits.hits[0].id);

    }


    // test list indices
    #[tokio::test]
    async fn test_list_indices() {
        let distant_client = DistantClient::new().await;
        let result = distant_client.list_indices().await.unwrap();

        // loop through the indices and print them
        for index in result {
            println!("Index: {:?}, docs: {:?}", index.index, index.docs_count);
        }
    }
}

