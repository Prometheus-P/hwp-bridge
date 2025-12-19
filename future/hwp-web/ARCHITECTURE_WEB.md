# Architecture — hwp-web (Planned)

This section was moved out of `docs/specs/ARCHITECTURE.md` for Option A.

## 4. Web Service Architecture

### 4.1 Request Flow

```
┌────────┐     ┌─────────────────────────────────────────────┐
│ Client │     │                  hwp-web                    │
└───┬────┘     └─────────────────────────────────────────────┘
    │                              │
    │  POST /api/convert           │
    │  (multipart/form-data)       │
    │─────────────────────────────▶│
    │                              │
    │                    ┌─────────▼─────────┐
    │                    │  Upload Handler   │
    │                    │  - File validation│
    │                    │  - Size check     │
    │                    └─────────┬─────────┘
    │                              │
    │                    ┌─────────▼─────────┐
    │                    │    hwp-core       │
    │                    │  - Parse HWP      │
    │                    │  - Convert HTML   │
    │                    └─────────┬─────────┘
    │                              │
    │                    ┌─────────▼─────────┐
    │                    │  Google Service   │
    │                    │  - OAuth check    │
    │                    │  - Upload to Drive│
    │                    └─────────┬─────────┘
    │                              │
    │  { docs_url, metadata }      │
    │◀─────────────────────────────│
    │                              │
```

### 4.2 Axum Router Structure

```rust
pub fn create_router() -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))

        // API routes
        .nest("/api",
            Router::new()
                .route("/convert", post(convert_handler))
                .route("/info", get(info_handler))
        )

        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB
}
```

---
