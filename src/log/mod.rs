use crate::ctx::Ctx;
use crate::web::rpc::RpcInfo;
use crate::web::{self, ClientError};
use crate::{Error, Result};
use axum::http::{Method, Uri};
use serde::Serialize;
use serde_json::{json, to_string_pretty, Value};
use serde_with::skip_serializing_none;
use std::time::SystemTime;
use tracing::debug;
use uuid::Uuid;


pub async fn log_request(
    uuid: Uuid,
    req_method: Method,
    uri: Uri,
    rpc_info: Option<&RpcInfo>,
    ctx: Option<Ctx>,
    web_error: Option<&web::Error>,
    client_error: Option<ClientError>,
) -> Result<()> {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let error_type = web_error.map(|e| e.as_ref().to_string());
    let error_data = serde_json::to_value(web_error)
        .ok()
        .and_then(|mut v| v.get_mut("data").map(|v| v.take()));

    // Create the RequestLogLine
    let log_line = RequestLogLine {
        uuid: uuid.to_string(),
        timestamp: timestamp.to_string(),

        req_path: uri.to_string(),
        req_method: req_method.to_string(),

        rpc_id: rpc_info.and_then(|rpc| rpc.id.as_ref().map(|id| id.to_string())),
        rpc_method: rpc_info.map(|rpc| rpc.method.to_string()),

        user_id: ctx.map(|c| c.user_id()),

        client_error_type: client_error.map(|e| e.as_ref().to_string()),

        error_type,
        error_data,
    };

    let log = json!(log_line);
    let log = to_string_pretty(&log).unwrap();
    debug!("log_request: {}", log);

    // TODO:  Send to logging service

    Ok(())
}


#[skip_serializing_none] // if Option is None, don't serialize it
#[derive(Serialize)]
struct RequestLogLine {
    uuid: String,      // uuid string formatted
    timestamp: String, // should be iso8601

    // user and context attributes
    user_id: Option<i64>,

    // http request attributes
    req_path: String,
    req_method: String,

    // RPC info
    rpc_id: Option<String>,
    rpc_method: Option<String>,

    // Error attributes
    client_error_type: Option<String>,
    error_type: Option<String>,
    error_data: Option<Value>,
}
