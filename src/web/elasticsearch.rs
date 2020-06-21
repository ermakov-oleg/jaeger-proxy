use std::collections::HashMap;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use surf::url::Url;
use log::{info};

use crate::web::models::{KeyValue, Log};

pub struct ESClient {
    host: Url,
    indexes: Vec<String>,
}


#[derive(Serialize, Deserialize, Debug)]
struct ESHits {
    total: u32,
    hits: Vec<ESDoc>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ESResult {
    hits: ESHits,
}

#[derive(Serialize, Deserialize, Debug)]
struct ESDoc {
    _source: ESLog,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ESLog {
    pub(crate) x_trace_id: Option<String>,

    #[serde(rename = "@timestamp")]
    pub(crate) timestamp: Option<String>,

    pub(crate) date: Option<String>,

    pub(crate) message: Option<String>,
    #[serde(flatten)]
    pub(crate) extra: HashMap<String, Value>,
}


impl From<ESLog> for Log {
    fn from(es: ESLog) -> Self {
        // es.timestamp format 2020-06-20T19:33:33.546Z

        // Indexing time, may be significantly late.
        let timestamp_datetime = NaiveDateTime::parse_from_str(
            &es.timestamp.clone().unwrap(),
            "%Y-%m-%dT%H:%M:%S.%fZ",
        ).unwrap();

        // Logging time
        // For some reason, it happens before the opening of the span.
        // let date_datetime = NaiveDateTime::parse_from_str(
        //     &es.date.clone().unwrap(),
        //     "%Y-%m-%dT%H:%M:%S.%f"
        // ).unwrap();
        //
        // let datetime = if date_datetime < timestamp_datetime { date_datetime } else { timestamp_datetime };

        Self {
            timestamp: timestamp_datetime.timestamp_millis() as u64 * 1000,
            fields: Some(vec![
                KeyValue {
                    key: "elk".to_string(),
                    type_value: Some("string".to_string()),
                    value: serde_json::to_value(&es).unwrap(),
                }
            ]),
        }
    }
}


impl ESClient {
    pub fn new(host: Url, indexes: Vec<String>) -> Self {
        Self {
            host,
            indexes,
        }
    }

    // todo: Error handling
    pub async fn get_logs(&self, trace_id: &String, from: u64, to: u64) -> Option<Vec<ESLog>> {
        let query = Self::get_query(trace_id, from, to);
        self.make_request(query).await
    }

    fn get_query(trace_id: &String, from: u64, to: u64) -> Value {
        json!({
          "query": {
            "bool": {
              "must": [
                {
                  "query_string": {
                    "query": trace_id
                  }
                },
                {
                  "range": {
                    "@timestamp": {
                      "gte": from,
                      "lte": to,
                      "format": "epoch_millis"
                    }
                  }
                }
              ]
            }
          }
        })
    }

    async fn make_request<Q: Serialize>(&self, query: Q) -> Option<Vec<ESLog>> {
        let index: String = self.indexes.join(",");
        let url = self.host.join(format!("{}/_search", index).as_str()).unwrap();

        info!("ES Search -> {}", &url);
        info!("{}", serde_json::to_string(&query).unwrap());

        let mut res = surf::post(&url).body_json(&query).unwrap().await.unwrap();

        info!("ES Response -> {} [{}]", &url, res.status());

        let result: ESResult = res.body_json().await.unwrap();


        Some(result.hits.hits.into_iter().map(|doc| doc._source).collect())
    }
}
