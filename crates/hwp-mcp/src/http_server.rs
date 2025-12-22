use std::{
    collections::{HashMap, VecDeque},
    convert::Infallible,
    env,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, SystemTime},
};

use tokio_stream::StreamExt;

use axum::{
    Json, Router,
    extract::{Query, State},
    http::{
        HeaderMap, HeaderValue, Method, StatusCode,
        header::{ACCEPT, CONTENT_TYPE, HeaderName, ORIGIN, SET_COOKIE},
    },
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use base64::Engine;
use tokio::sync::{RwLock, mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::tools;

// ─────────────────────────────────────────────────────────────────────────────
// JSON-RPC types (local definitions for http transport)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

fn mcp_error(id: Option<serde_json::Value>, code: i32, message: String) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(JsonRpcError {
            code,
            message,
            data: None,
        }),
    }
}

fn mcp_response(id: Option<serde_json::Value>, result: serde_json::Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result),
        error: None,
    }
}

pub const PROTOCOL_V2025_03_26: &str = "2025-03-26";
pub const PROTOCOL_V2025_06_18: &str = "2025-06-18";

type QueryMap = HashMap<String, String>;

#[derive(Clone)]
struct AppState {
    sessions: Arc<RwLock<HashMap<String, McpSession>>>,
}

#[derive(Clone)]
struct McpSession {
    created_at: SystemTime,
    log_tx: mpsc::UnboundedSender<ServerLogEvent>,
    protocol_version: String,
    // Very small replay buffer for resumable SSE (best-effort).
    replay: Arc<RwLock<VecDeque<(u64, String)>>>,
    next_event_id: Arc<RwLock<u64>>,
}

#[derive(Clone, Debug)]
struct ServerLogEvent {
    level: &'static str,
    message: String,
}

#[derive(Clone, Debug)]
struct RuntimeLimits {
    max_file_bytes: usize,
    max_records: usize,
    max_sections: usize,
}

impl Default for RuntimeLimits {
    fn default() -> Self {
        Self {
            max_file_bytes: 10 * 1024 * 1024, // 10MB
            max_records: 100_000,
            max_sections: 50_000,
        }
    }
}

#[derive(Clone, Debug)]
struct AllowedOrigins {
    any: bool,
    list: Vec<String>,
}

impl AllowedOrigins {
    fn from_env() -> Self {
        let raw = env::var("HWP_ALLOWED_ORIGINS").unwrap_or_else(|_| "*".to_string());
        let raw = raw.trim();
        if raw.is_empty() || raw == "*" {
            return Self {
                any: true,
                list: vec!["*".to_string()],
            };
        }
        let list = raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        Self { any: false, list }
    }

    fn matches(&self, origin: &str) -> bool {
        if self.any {
            return true;
        }
        self.list.iter().any(|o| o == origin)
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_millis()
}

fn decode_config_json(query: &QueryMap) -> Option<serde_json::Value> {
    let b64 = query.get("config")?.trim();
    if b64.is_empty() {
        return None;
    }

    // Smithery commonly passes a URL-safe base64 (sometimes padded, sometimes not).
    let std_engine = base64::engine::general_purpose::STANDARD;
    let url_engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;

    let bytes = std_engine
        .decode(b64)
        .or_else(|_| url_engine.decode(b64))
        .ok()?;

    let txt = String::from_utf8(bytes).ok()?;
    let v: serde_json::Value = serde_json::from_str(&txt).ok()?;
    Some(v)
}

fn json_pointer_for(key_path: &str) -> String {
    // "limits.maxFileBytes" -> "/limits/maxFileBytes"
    let parts = key_path.split('.').collect::<Vec<_>>();
    let mut out = String::new();
    for p in parts {
        out.push('/');
        out.push_str(p);
    }
    out
}

fn get_str_param(query: &QueryMap, cfg: &Option<serde_json::Value>, key: &str) -> Option<String> {
    if let Some(v) = query.get(key) {
        if !v.trim().is_empty() {
            return Some(v.trim().to_string());
        }
    }
    let cfg = cfg.as_ref()?;
    let ptr = json_pointer_for(key);
    cfg.pointer(&ptr)
        .and_then(|v| v.as_str().map(|s| s.to_string()))
}

fn get_usize_param(query: &QueryMap, cfg: &Option<serde_json::Value>, key: &str) -> Option<usize> {
    if let Some(v) = query.get(key) {
        if let Ok(n) = v.trim().parse::<usize>() {
            return Some(n);
        }
    }
    let cfg = cfg.as_ref()?;
    let ptr = json_pointer_for(key);
    let v = cfg.pointer(&ptr)?;
    if let Some(n) = v.as_u64() {
        return Some(n as usize);
    }
    if let Some(s) = v.as_str() {
        return s.trim().parse::<usize>().ok();
    }
    None
}

fn get_string_list_param(
    query: &QueryMap,
    cfg: &Option<serde_json::Value>,
    key: &str,
) -> Option<Vec<String>> {
    // query: comma-separated
    if let Some(v) = query.get(key) {
        let list = v
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        if !list.is_empty() {
            return Some(list);
        }
    }

    let cfg = cfg.as_ref()?;
    let ptr = json_pointer_for(key);
    let v = cfg.pointer(&ptr)?;

    if let Some(arr) = v.as_array() {
        let list = arr
            .iter()
            .filter_map(|x| x.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>();
        if !list.is_empty() {
            return Some(list);
        }
    }

    if let Some(s) = v.as_str() {
        let list = s
            .split(',')
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>();
        if !list.is_empty() {
            return Some(list);
        }
    }

    None
}

fn parse_limits(query: &QueryMap) -> RuntimeLimits {
    let defaults = RuntimeLimits::default();
    let cfg = decode_config_json(query);

    let max_file_bytes = get_usize_param(query, &cfg, "limits.maxFileBytes")
        .or_else(|| get_usize_param(query, &cfg, "maxFileBytes"))
        .unwrap_or(defaults.max_file_bytes)
        .clamp(256 * 1024, 256 * 1024 * 1024); // 256KB .. 256MB

    let max_records = get_usize_param(query, &cfg, "limits.maxRecords")
        .or_else(|| get_usize_param(query, &cfg, "maxRecords"))
        .unwrap_or(defaults.max_records)
        .clamp(1_000, 5_000_000);

    let max_sections = get_usize_param(query, &cfg, "limits.maxSections")
        .or_else(|| get_usize_param(query, &cfg, "maxSections"))
        .unwrap_or(defaults.max_sections)
        .clamp(1_000, 5_000_000);

    RuntimeLimits {
        max_file_bytes,
        max_records,
        max_sections,
    }
}

fn parse_allowed_origins(query: &QueryMap) -> AllowedOrigins {
    let cfg = decode_config_json(query);

    // priority: config -> env
    if let Some(list) = get_string_list_param(query, &cfg, "security.allowedOrigins")
        .or_else(|| get_string_list_param(query, &cfg, "allowedOrigins"))
    {
        if list.len() == 1 && list[0] == "*" {
            return AllowedOrigins {
                any: true,
                list: vec!["*".to_string()],
            };
        }
        return AllowedOrigins { any: false, list };
    }

    AllowedOrigins::from_env()
}

fn validate_origin(headers: &HeaderMap, allowed: &AllowedOrigins) -> Result<(), Response> {
    let Some(origin) = headers.get(ORIGIN) else {
        // Many non-browser clients won't send Origin.
        return Ok(());
    };

    let Ok(origin_str) = origin.to_str() else {
        return Err((StatusCode::FORBIDDEN, "Invalid Origin header").into_response());
    };

    if allowed.matches(origin_str) {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            format!("Origin not allowed: {origin_str}"),
        )
            .into_response())
    }
}

fn header_get_ci(headers: &HeaderMap, name: &str) -> Option<String> {
    // Case-insensitive lookup by iterating.
    for (k, v) in headers.iter() {
        if k.as_str().eq_ignore_ascii_case(name) {
            if let Ok(s) = v.to_str() {
                return Some(s.to_string());
            }
        }
    }
    None
}

fn accept_has(accept: &str, needle: &str) -> bool {
    accept.split(',').any(|part| {
        part.trim()
            .to_ascii_lowercase()
            .starts_with(&needle.to_ascii_lowercase())
    })
}

fn add_mcp_headers(resp: &mut Response, protocol_version: &str, session_id: Option<&str>) {
    let headers = resp.headers_mut();

    if let Ok(v) = HeaderValue::from_str(protocol_version) {
        headers.insert("Mcp-Protocol-Version", v);
    }
    if let Some(sid) = session_id {
        if let Ok(v) = HeaderValue::from_str(sid) {
            headers.insert("Mcp-Session-Id", v);
        }
    }
}

pub async fn serve() -> anyhow::Result<()> {
    let state = AppState {
        sessions: Arc::new(RwLock::new(HashMap::new())),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .expose_headers([
            HeaderName::from_static("mcp-session-id"),
            HeaderName::from_static("mcp-protocol-version"),
        ]);

    let mut app = Router::new()
        .route("/", get(|| async move { Redirect::to("/health") }))
        .route("/health", get(health))
        .route("/mcp", get(mcp_get).post(mcp_post).delete(mcp_delete))
        .with_state(state);

    // Dev UI is feature-gated and runtime-gated.
    #[cfg(feature = "dev-ui")]
    {
        if env::var("HWP_DEV_UI").ok().as_deref() == Some("1") {
            app = app.merge(dev_ui_router());
        }
    }

    app = app.layer(cors);

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8081);

    let addr: SocketAddr = format!("0.0.0.0:{port}").parse()?;

    tracing::info!("HWP MCP HTTP server listening on http://{addr}/mcp");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> impl IntoResponse {
    let body = serde_json::json!({
        "ok": true,
        "service": "hwp-mcp",
        "transport": "streamable-http",
        "endpoint": "/mcp",
        "protocols": [PROTOCOL_V2025_03_26, PROTOCOL_V2025_06_18],
        "now_ms": now_ms(),
    });
    (StatusCode::OK, Json(body))
}

fn require_sse_accept(headers: &HeaderMap) -> Result<(), Response> {
    let accept = headers
        .get(ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if accept_has(accept, "text/event-stream") {
        Ok(())
    } else {
        Err((StatusCode::BAD_REQUEST, "Missing Accept: text/event-stream").into_response())
    }
}

fn ensure_json_content_type(headers: &HeaderMap) -> Result<(), Response> {
    let content_type = headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // allow charset
    if content_type
        .to_ascii_lowercase()
        .starts_with("application/json")
    {
        Ok(())
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            "Content-Type must be application/json",
        )
            .into_response())
    }
}

fn ensure_post_accept(headers: &HeaderMap) -> Result<(), Response> {
    let accept = headers
        .get(ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Spec: must list both; be tolerant but enforce presence of application/json.
    if accept_has(accept, "application/json") {
        Ok(())
    } else {
        Err((StatusCode::BAD_REQUEST, "Missing Accept: application/json").into_response())
    }
}

async fn mcp_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<QueryMap>,
) -> Response {
    let allowed = parse_allowed_origins(&query);
    if let Err(resp) = validate_origin(&headers, &allowed) {
        return resp;
    }

    if let Err(resp) = require_sse_accept(&headers) {
        return resp;
    }

    // Optional session (to resume logs).
    let sid = header_get_ci(&headers, "Mcp-Session-Id").or_else(|| query.get("sid").cloned());

    let Some(sid) = sid else {
        return (
            StatusCode::BAD_REQUEST,
            "Missing Mcp-Session-Id header (or sid query)",
        )
            .into_response();
    };

    let sessions = state.sessions.read().await;
    let Some(session) = sessions.get(&sid) else {
        return (StatusCode::NOT_FOUND, "Session not found").into_response();
    };

    // Support a tiny best-effort replay via Last-Event-ID.
    let last_event_id =
        header_get_ci(&headers, "Last-Event-ID").and_then(|s| s.parse::<u64>().ok());

    let (tx, rx) = mpsc::unbounded_channel::<String>();

    // 1) Replay buffered events (if any)
    if let Some(last) = last_event_id {
        let replay = session.replay.read().await;
        for (id, payload) in replay.iter() {
            if *id > last {
                let _ = tx.send(format!("id: {id}\ndata: {payload}\n\n"));
            }
        }
    }

    // 2) Stream live logs (converted to SSE events)
    let (live_tx, mut live_rx) = mpsc::unbounded_channel::<ServerLogEvent>();
    // Clone session log sender? We don't have log broadcast; we'll just attach this stream by replacing sender?
    // Instead: we keep the session log_tx as a single sender, but we can tee by sending to both:
    // For simplicity: we create an ephemeral forwarder: we push a "connected" message into this stream only.
    let _ = live_tx.send(ServerLogEvent {
        level: "info",
        message: format!("SSE connected (sid={sid})"),
    });

    let replay_buf = session.replay.clone();
    let next_id = session.next_event_id.clone();
    tokio::spawn(async move {
        while let Some(ev) = live_rx.recv().await {
            let id = {
                let mut guard = next_id.write().await;
                *guard += 1;
                *guard
            };

            let payload = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "notifications/message",
                "params": {
                    "level": ev.level,
                    "data": ev.message,
                }
            })
            .to_string();

            // buffer (tiny)
            {
                let mut r = replay_buf.write().await;
                r.push_back((id, payload.clone()));
                while r.len() > 128 {
                    r.pop_front();
                }
            }

            let _ = tx.send(format!("id: {id}\ndata: {payload}\n\n"));
        }
    });

    let stream = UnboundedReceiverStream::new(rx).map(Ok::<_, Infallible>);

    let mut resp = Response::new(axum::body::Body::from_stream(stream));
    *resp.status_mut() = StatusCode::OK;
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("text/event-stream"));

    // Echo protocol + session headers for clients.
    add_mcp_headers(&mut resp, &session.protocol_version, Some(&sid));

    resp
}

async fn mcp_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<QueryMap>,
) -> Response {
    let allowed = parse_allowed_origins(&query);
    if let Err(resp) = validate_origin(&headers, &allowed) {
        return resp;
    }

    let sid = header_get_ci(&headers, "Mcp-Session-Id").or_else(|| query.get("sid").cloned());

    let Some(sid) = sid else {
        return (
            StatusCode::BAD_REQUEST,
            "Missing Mcp-Session-Id header (or sid query)",
        )
            .into_response();
    };

    let mut sessions = state.sessions.write().await;
    if sessions.remove(&sid).is_some() {
        StatusCode::NO_CONTENT.into_response()
    } else {
        (StatusCode::NOT_FOUND, "Session not found").into_response()
    }
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum PostMode {
    Auto,
    Json,
    Sse,
}

impl Default for PostMode {
    fn default() -> Self {
        Self::Auto
    }
}

async fn mcp_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<QueryMap>,
    body: axum::body::Bytes,
) -> Response {
    let allowed = parse_allowed_origins(&query);
    if let Err(resp) = validate_origin(&headers, &allowed) {
        return resp;
    }

    if let Err(resp) = ensure_json_content_type(&headers) {
        return resp;
    }

    if let Err(resp) = ensure_post_accept(&headers) {
        return resp;
    }

    let limits = parse_limits(&query);

    // Try parse JSON (could be single or batch)
    let parsed_json: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("Invalid JSON body: {e}")).into_response();
        }
    };

    let session_id =
        header_get_ci(&headers, "Mcp-Session-Id").or_else(|| query.get("sid").cloned());

    // Determine request list
    let mut messages: Vec<JsonRpcMessage> = vec![];
    let is_batch = parsed_json.is_array();

    if is_batch {
        // Will validate against protocol later.
        match serde_json::from_value::<Vec<JsonRpcMessage>>(parsed_json.clone()) {
            Ok(v) => messages = v,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    format!("Invalid JSON-RPC batch: {e}"),
                )
                    .into_response();
            }
        }
    } else {
        match serde_json::from_value::<JsonRpcMessage>(parsed_json.clone()) {
            Ok(v) => messages = vec![v],
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    format!("Invalid JSON-RPC message: {e}"),
                )
                    .into_response();
            }
        }
    }

    // Figure out if this is initialize.
    let is_initialize = messages.iter().any(|m| match m {
        JsonRpcMessage::Request(r) => r.method == "initialize",
        _ => false,
    });

    // Requested protocol from header (if any)
    let header_proto = header_get_ci(&headers, "Mcp-Protocol-Version")
        .or_else(|| header_get_ci(&headers, "MCP-Protocol-Version"));

    // Determine effective protocol for this request.
    #[allow(unused_assignments)]
    let mut effective_protocol = PROTOCOL_V2025_06_18.to_string();

    if is_initialize {
        // Parse from initialize params if present.
        let requested = messages.iter().find_map(|m| match m {
            JsonRpcMessage::Request(req) if req.method == "initialize" => req
                .params
                .as_ref()
                .and_then(|p| p.get("protocolVersion"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            _ => None,
        });

        let requested = requested.unwrap_or_else(|| PROTOCOL_V2025_06_18.to_string());
        if requested != PROTOCOL_V2025_06_18 && requested != PROTOCOL_V2025_03_26 {
            let mut resp = (
                StatusCode::BAD_REQUEST,
                Json(mcp_error(
                    None,
                    -32000,
                    format!(
                        "Unsupported protocol version. Supported: {PROTOCOL_V2025_03_26}, {PROTOCOL_V2025_06_18}. Requested: {requested}"
                    ),
                )),
            )
                .into_response();
            add_mcp_headers(&mut resp, PROTOCOL_V2025_06_18, None);
            return resp;
        }
        effective_protocol = requested;
    } else {
        // Use session protocol if exists; else fall back to header; else assume 03-26.
        if let Some(sid) = session_id.as_deref() {
            let sessions = state.sessions.read().await;
            if let Some(sess) = sessions.get(sid) {
                effective_protocol = sess.protocol_version.clone();
            } else {
                return (StatusCode::NOT_FOUND, "Session not found").into_response();
            }
        } else if let Some(h) = header_proto.clone() {
            effective_protocol = h;
        } else {
            effective_protocol = PROTOCOL_V2025_03_26.to_string();
        }

        // Validate header protocol if present.
        if let Some(h) = header_proto {
            if h != effective_protocol {
                return (
                    StatusCode::BAD_REQUEST,
                    format!(
                        "Mcp-Protocol-Version mismatch. Session expects {effective_protocol}, got {h}"
                    ),
                )
                    .into_response();
            }
        }
    }

    // Enforce batch rule for newer protocol.
    if is_batch && effective_protocol == PROTOCOL_V2025_06_18 {
        let mut resp = (
            StatusCode::BAD_REQUEST,
            Json(mcp_error(
                None,
                -32600,
                "Batch requests are not supported for protocol 2025-06-18".to_string(),
            )),
        )
            .into_response();
        add_mcp_headers(&mut resp, &effective_protocol, session_id.as_deref());
        return resp;
    }

    // Create session on initialize and assign protocol.
    let sid_for_request = if is_initialize {
        let sid = Uuid::new_v4().to_string();
        let (log_tx, _log_rx) = mpsc::unbounded_channel::<ServerLogEvent>();
        let sess = McpSession {
            created_at: SystemTime::now(),
            log_tx,
            protocol_version: effective_protocol.clone(),
            replay: Arc::new(RwLock::new(VecDeque::with_capacity(128))),
            next_event_id: Arc::new(RwLock::new(0)),
        };
        let mut sessions = state.sessions.write().await;
        sessions.insert(sid.clone(), sess);
        Some(sid)
    } else {
        session_id
    };

    // Determine if mode is forced.
    let mode = query
        .get("mode")
        .and_then(|s| serde_json::from_str::<PostMode>(&format!("\"{}\"", s)).ok())
        .unwrap_or_default();

    // Process requests.
    let mut out: Vec<JsonRpcResponse> = Vec::new();

    for msg in messages {
        match msg {
            JsonRpcMessage::Request(req) => {
                if req.method == "initialize" {
                    out.push(mcp_response(
                        req.id.clone(),
                        serde_json::json!({
                            "protocolVersion": effective_protocol,
                            "serverInfo": {
                                "name": "hwp-mcp",
                                "version": env!("CARGO_PKG_VERSION"),
                            },
                            "capabilities": {
                                "tools": { "listChanged": false },
                            }
                        }),
                    ));
                    continue;
                }

                // Need session for all subsequent requests.
                let Some(sid) = sid_for_request.as_deref() else {
                    out.push(mcp_error(
                        req.id.clone(),
                        -32001,
                        "Missing session. Initialize first.".to_string(),
                    ));
                    continue;
                };

                match handle_rpc_request(&state, &sid, req, limits.clone()).await {
                    Ok(Some(resp)) => out.push(resp),
                    Ok(None) => {}
                    Err(err_resp) => out.push(err_resp),
                }
            }
            JsonRpcMessage::Notification(_n) => {
                // no-op
            }
            JsonRpcMessage::Response(_r) => {
                // no-op
            }
        }
    }

    // Decide response media type.
    let want_sse = match mode {
        PostMode::Sse => true,
        PostMode::Json => false,
        PostMode::Auto => {
            // If multiple responses, JSON array; else JSON.
            false
        }
    };

    if out.is_empty() {
        let mut resp = StatusCode::ACCEPTED.into_response();
        add_mcp_headers(&mut resp, &effective_protocol, sid_for_request.as_deref());
        return resp;
    }

    if want_sse {
        // SSE stream that includes the response and then closes.
        let (tx, rx) = mpsc::unbounded_channel::<String>();

        let sid_hdr = sid_for_request.clone();
        let proto_hdr = effective_protocol.clone();
        tokio::spawn(async move {
            for r in out {
                let payload = serde_json::to_string(&r).unwrap_or_else(|_| "{}".to_string());
                let _ = tx.send(format!("data: {payload}\n\n"));
            }
            drop(tx);
            let _ = sid_hdr;
            let _ = proto_hdr;
        });

        let stream = UnboundedReceiverStream::new(rx).map(Ok::<_, Infallible>);
        let mut resp = Response::new(axum::body::Body::from_stream(stream));
        *resp.status_mut() = StatusCode::OK;
        resp.headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("text/event-stream"));

        add_mcp_headers(&mut resp, &effective_protocol, sid_for_request.as_deref());

        resp
    } else {
        // JSON response
        let body = if out.len() == 1 {
            serde_json::to_value(&out[0]).unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::to_value(&out).unwrap_or_else(|_| serde_json::json!([]))
        };

        let mut resp = (StatusCode::OK, Json(body)).into_response();
        add_mcp_headers(&mut resp, &effective_protocol, sid_for_request.as_deref());
        resp
    }
}

async fn handle_rpc_request(
    state: &AppState,
    sid: &str,
    req: JsonRpcRequest,
    limits: RuntimeLimits,
) -> Result<Option<JsonRpcResponse>, JsonRpcResponse> {
    // Session existence check (also enables 404 semantics).
    {
        let sessions = state.sessions.read().await;
        if !sessions.contains_key(sid) {
            return Err(mcp_error(
                req.id.clone(),
                -32004,
                "Session not found or expired".to_string(),
            ));
        }
    }

    match req.method.as_str() {
        "tools/list" => {
            let list = tools::list_tools();
            Ok(Some(mcp_response(
                req.id,
                serde_json::json!({ "tools": list }),
            )))
        }
        "tools/call" => {
            let Some(params) = req.params else {
                return Err(mcp_error(req.id, -32602, "Missing params".to_string()));
            };

            let name = params
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    mcp_error(req.id.clone(), -32602, "Missing params.name".to_string())
                })?
                .to_string();

            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));

            match tools::call_tool(
                &name,
                arguments,
                limits.max_file_bytes,
                limits.max_records,
                limits.max_sections,
            )
            .await
            {
                Ok(result) => Ok(Some(mcp_response(req.id, result))),
                Err(e) => Err(mcp_error(req.id, -32010, format!("Tool error: {e}"))),
            }
        }
        _ => Err(mcp_error(
            req.id,
            -32601,
            format!("Method not found: {}", req.method),
        )),
    }
}

#[cfg(feature = "dev-ui")]
mod dev_ui {
    use super::*;

    pub(super) fn dev_ui_router() -> Router<()> {
        Router::new()
            .route("/__dev/ui", get(ui_index))
            .route("/__dev/login", get(ui_login_get).post(ui_login_post))
            .route("/__dev/session", get(ui_session))
    }

    fn is_authed(headers: &HeaderMap) -> bool {
        headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .map(|c| c.contains("hwp_dev_ui=1"))
            .unwrap_or(false)
    }

    async fn ui_login_get(headers: HeaderMap) -> Response {
        if is_authed(&headers) {
            return Redirect::to("/__dev/ui").into_response();
        }

        let html = r#"<!doctype html>
<html>
<head>
  <meta charset='utf-8'/>
  <meta name='viewport' content='width=device-width,initial-scale=1'/>
  <title>HWP MCP Dev UI Login</title>
  <style>
    body{font-family:system-ui,-apple-system,Segoe UI,Roboto,Arial;padding:24px;background:#0b0b0c;color:#e7e7ea}
    .card{max-width:420px;margin:0 auto;background:#141416;border:1px solid #2a2a2f;border-radius:14px;padding:18px}
    input{width:100%;padding:12px;border-radius:10px;border:1px solid #333;background:#0f0f12;color:#fff}
    button{margin-top:12px;width:100%;padding:12px;border-radius:10px;border:0;background:#3b82f6;color:#fff;font-weight:700}
    .hint{opacity:.75;font-size:13px;margin-top:10px}
  </style>
</head>
<body>
  <div class='card'>
    <h2>Dev UI Login</h2>
    <form method='post' action='/__dev/login'>
      <input name='token' placeholder='HWP_DEV_UI_TOKEN' autofocus />
      <button type='submit'>Login</button>
    </form>
    <div class='hint'>Set env: HWP_DEV_UI=1 and HWP_DEV_UI_TOKEN=... then login.</div>
  </div>
</body>
</html>"#;

        Html(html).into_response()
    }

    #[derive(serde::Deserialize)]
    struct LoginForm {
        token: String,
    }

    async fn ui_login_post(
        axum::extract::Form(form): axum::extract::Form<LoginForm>,
    ) -> Response {
        let token = env::var("HWP_DEV_UI_TOKEN").unwrap_or_default();
        if token.is_empty() {
            return (
                StatusCode::PRECONDITION_FAILED,
                "HWP_DEV_UI_TOKEN is not set",
            )
                .into_response();
        }

        if form.token.trim() == token {
            let mut resp = Redirect::to("/__dev/ui").into_response();
            resp.headers_mut().insert(
                SET_COOKIE,
                HeaderValue::from_static("hwp_dev_ui=1; Path=/; HttpOnly; SameSite=Lax"),
            );
            return resp;
        }

        (StatusCode::UNAUTHORIZED, "Invalid token").into_response()
    }

    async fn ui_session(headers: HeaderMap) -> Response {
        if !is_authed(&headers) {
            return Redirect::to("/__dev/login").into_response();
        }

        // Create a new session by calling initialize.
        // Note: Dev UI calls the public /mcp endpoint.
        let init = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "ui-init",
            "method": "initialize",
            "params": {
                "protocolVersion": PROTOCOL_V2025_06_18,
                "clientInfo": {"name": "hwp-dev-ui", "version": "0.1"},
                "capabilities": {"tools": {}}
            }
        });

        // We cannot call internal handlers directly; user should use curl/playground.
        // Keep endpoint as a helper.
        let body = serde_json::json!({
            "ok": true,
            "note": "Dev UI uses browser fetch to /mcp. Use the UI page to run initialize/tools.",
            "exampleInitialize": init,
        });

        (StatusCode::OK, Json(body)).into_response()
    }

    async fn ui_index(headers: HeaderMap) -> Response {
        if !is_authed(&headers) {
            return Redirect::to("/__dev/login").into_response();
        }

        let html = r#"<!doctype html>
<html>
<head>
  <meta charset='utf-8'/>
  <meta name='viewport' content='width=device-width,initial-scale=1'/>
  <title>HWP MCP Dev UI</title>
  <style>
    :root{color-scheme:dark}
    body{font-family:system-ui,-apple-system,Segoe UI,Roboto,Arial;background:#0b0b0c;color:#e7e7ea;margin:0}
    header{padding:14px 18px;border-bottom:1px solid #232329;display:flex;gap:12px;align-items:center}
    header b{font-size:15px}
    .wrap{display:grid;grid-template-columns:420px 1fr;gap:14px;padding:14px}
    .card{background:#141416;border:1px solid #232329;border-radius:14px;padding:14px}
    textarea,input,select{width:100%;box-sizing:border-box;padding:10px;border-radius:10px;border:1px solid #333;background:#0f0f12;color:#fff}
    button{padding:10px 12px;border-radius:10px;border:0;background:#3b82f6;color:#fff;font-weight:700;cursor:pointer}
    button.secondary{background:#374151}
    .row{display:flex;gap:8px;align-items:center}
    .row > *{flex:1}
    pre{white-space:pre-wrap;word-break:break-word;background:#0f0f12;border:1px solid #2a2a2f;border-radius:12px;padding:12px;min-height:220px}
    .muted{opacity:.7;font-size:12px}
    .warn{background:#2a1313;border:1px solid #4b1e1e;color:#ffb4b4;padding:10px;border-radius:12px;margin:10px 0;font-size:13px}
  </style>
</head>
<body>
<header>
  <b>HWP MCP Dev UI</b>
  <span class='muted'>/mcp streamable-http · protocol 2025-06-18</span>
</header>
<div class='wrap'>
  <div class='card'>
    <div class='row'>
      <button id='btnInit'>Initialize</button>
      <button class='secondary' id='btnTools'>tools/list</button>
    </div>
    <div style='height:10px'></div>
    <label class='muted'>Tool</label>
    <select id='toolName'>
      <option value='hwp.inspect'>hwp.inspect</option>
      <option value='hwp.extract'>hwp.extract</option>
      <option value='hwp.to_markdown'>hwp.to_markdown</option>
      <option value='hwp.to_json'>hwp.to_json</option>
    </select>
    <div style='height:10px'></div>
    <div class='warn'>HWPX (.hwpx) is not supported yet. Supported: HWP v5 (.hwp, OLE/CFB).</div>
    <label class='muted'>Pick .hwp (inline base64)</label>
    <div class='row'>
      <input id='filePick' type='file' accept='.hwp' />
      <button class='secondary' id='btnLoadInline'>Load into args</button>
    </div>
    <div style='height:10px'></div>
    <label class='muted'>Arguments (JSON)</label>
    <textarea id='args' rows='12'>{
  "file": {
    "name": "sample.hwp",
    "content": ""
  }
}</textarea>
    <div style='height:10px'></div>
    <div class='row'>
      <button id='btnCall'>tools/call</button>
      <button class='secondary' id='btnHealth'>/health</button>
    </div>
    <div style='height:10px'></div>
    <div class='muted'>Session: <span id='sid'>(none)</span></div>
    <div class='muted'>Protocol: <span id='proto'>(none)</span></div>
  </div>
  <div class='card'>
    <label class='muted'>Output</label>
    <pre id='out'></pre>
  </div>
</div>
<script>
  let sid = null;
  let proto = null;

  function setOut(obj){
    const out = document.getElementById('out');
    out.textContent = typeof obj === 'string' ? obj : JSON.stringify(obj, null, 2);
    document.getElementById('sid').textContent = sid || '(none)';
    document.getElementById('proto').textContent = proto || '(none)';
  }


  function unsupportedHwpx(inputName){
    const msg = "UNSUPPORTED_FORMAT: HWPX (.hwpx) is not supported yet. Supported: HWP v5 (.hwp, OLE/CFB).";
    return {
      error: {
        code: "UNSUPPORTED_FORMAT",
        message: msg,
        details: { format: "hwpx", supported: ["hwp"], input: inputName || "" }
      }
    };
  }

  function isHwpxName(name){
    return (name || "").toLowerCase().endsWith(".hwpx");
  }

  function detectHwpxFromArgs(args){
    const f = (args && args.file) || {};
    const n = (f.name || f.path || "");
    return isHwpxName(n);
  }

  function fileToBase64(file){
    return new Promise((resolve, reject) => {
      const r = new FileReader();
      r.onload = () => {
        const s = String(r.result || "");
        const b64 = s.includes(",") ? s.split(",")[1] : "";
        resolve(b64);
      };
      r.onerror = () => reject(r.error || new Error("FileReader error"));
      r.readAsDataURL(file);
    });
  }

  async function postMcp(payload){
    const headers = {
      'Content-Type':'application/json',
      'Accept':'application/json, text/event-stream',
    };
    if (sid) headers['Mcp-Session-Id'] = sid;
    if (proto) headers['Mcp-Protocol-Version'] = proto;

    const res = await fetch('/mcp', {method:'POST', headers, body: JSON.stringify(payload)});
    const sidHdr = res.headers.get('Mcp-Session-Id') || res.headers.get('mcp-session-id');
    const protoHdr = res.headers.get('Mcp-Protocol-Version') || res.headers.get('mcp-protocol-version');
    if (sidHdr) sid = sidHdr;
    if (protoHdr) proto = protoHdr;

    const ct = (res.headers.get('content-type')||'').toLowerCase();
    if (ct.includes('text/event-stream')){
      const text = await res.text();
      // naive parse: show raw
      setOut(text);
      return;
    }
    const json = await res.json().catch(()=>({raw: res.status}));
    setOut(json);
  }

  document.getElementById('btnInit').onclick = async () => {
    sid = null;
    proto = '2025-06-18';
    await postMcp({
      jsonrpc:'2.0',
      id:'init-1',
      method:'initialize',
      params:{
        protocolVersion:'2025-06-18',
        clientInfo:{name:'hwp-dev-ui', version:'0.1'},
        capabilities:{tools:{}}
      }
    });
  };

  document.getElementById('btnTools').onclick = async () => {
    await postMcp({jsonrpc:'2.0', id:'tools-1', method:'tools/list'});
  };

  document.getElementById('btnCall').onclick = async () => {
    const name = document.getElementById('toolName').value;
    let args = {};
    try{
      args = JSON.parse(document.getElementById('args').value || '{}');
    } catch(e){
      setOut('Invalid JSON in arguments');
      return;
    }
    if (detectHwpxFromArgs(args)) {
      const f = (args && args.file) || {};
      setOut(unsupportedHwpx(f.name || f.path || ''));
      return;
    }
    await postMcp({jsonrpc:'2.0', id:'call-1', method:'tools/call', params:{name, arguments: args}});
  };




  document.getElementById('btnLoadInline').onclick = async () => {
    const input = document.getElementById('filePick');
    const f = input && input.files && input.files[0];
    if (!f) { setOut('Pick a .hwp file first'); return; }
    if (isHwpxName(f.name)) { setOut(unsupportedHwpx(f.name)); return; }
    try{
      const b64 = await fileToBase64(f);
      const args = { file: { name: f.name, content: b64 } };
      document.getElementById('args').value = JSON.stringify(args, null, 2);
      setOut({ok:true, note:'Loaded inline base64 into args', name: f.name, bytesApprox: b64.length});
    } catch(e){
      setOut({error:'Failed to read file', detail: String(e)});
    }
  };

  document.getElementById('btnHealth').onclick = async () => {
    const res = await fetch('/health');
    const json = await res.json();
    setOut(json);
  };
</script>
</body>
</html>"#;

        Html(html).into_response()
    }
}

#[cfg(feature = "dev-ui")]
use dev_ui::dev_ui_router;
