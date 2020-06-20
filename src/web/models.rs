use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    code: i32,
    msg: String,
    #[serde(rename = "traceID")]
    trace_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Reference {
    #[serde(rename = "refType")]
    ref_type: String,
    #[serde(rename = "traceID")]
    trace_id: String,
    #[serde(rename = "spanID")]
    span_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyValue {
    key: String,
    #[serde(rename = "type")]
    type_value: Option<String>,
    value: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Log {
    timestamp: u64,
    fields: Option<Vec<KeyValue>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Span {
    #[serde(rename = "traceID")]
    trace_id: String,
    #[serde(rename = "spanID")]
    span_id: String,
    #[serde(rename = "parentSpanID")]
    parent_span_id: Option<String>,
    flags: Option<u32>,
    #[serde(rename = "operationName")]
    operation_name: String,
    references: Option<Vec<Reference>>,
    #[serde(rename = "startTime")]
    start_time: u64,
    duration: u64,
    tags: Option<Vec<KeyValue>>,
    logs: Option<Vec<Log>>,
    #[serde(rename = "processID")]
    process_id: Option<String>,
    process: Option<Process>,
    warnings: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Process {
    #[serde(rename = "serviceName")]
    service_name: String,
    tags: Option<Vec<KeyValue>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Trace {
    #[serde(rename = "traceID")]
    trace_id: String,
    spans: Option<Vec<Span>>,
    processes: HashMap<String, Process>,
    warnings: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTraceResponse {
    data: Option<Vec<Trace>>,
    errors: Option<Vec<Error>>,
}
