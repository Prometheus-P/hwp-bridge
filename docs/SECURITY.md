# Security Notes

This project processes untrusted binary documents.

## Threat model
- Malformed documents causing panic / DoS
- Zip-bombs / decompression bombs
- Path traversal (when writing extracted assets)
- Memory spikes / OOM

## Current mitigations
- MCP server enforces max input size via `HWP_MAX_FILE_BYTES`
- HTTP transport includes a request body limit (derived from `HWP_MAX_FILE_BYTES`)
- Decompression + record-count limits via `HWP_MAX_DECOMPRESSED_BYTES_PER_SECTION` and `HWP_MAX_RECORDS_PER_SECTION`
- Path-based input is **disabled by default**; enable only for local use via `HWP_ALLOW_PATH_INPUT=1` (+ optional `HWP_PATH_BASEDIR`)

## Dev UI
An optional local dev UI exists behind a shared token + short-lived session cookie.

**Hard gates (both required):**
- Compile-time: `cargo run -p hwp-mcp --features dev-ui`
- Runtime: `HWP_DEV_UI=1` and `HWP_DEV_UI_TOKEN=...`

It is **disabled by default** and must not be enabled in public deployments.

## TODO
- Decompression size limits (per section) ✅ implemented (see HWP_MAX_DECOMPRESSED_BYTES_PER_SECTION)
- Timeouts per conversion
- Fuzzing harness for record parsing


## Limits knobs
- `HWP_MAX_DECOMPRESSED_BYTES_PER_SECTION`
- `HWP_MAX_RECORDS_PER_SECTION`


#[cfg(test)]
mod tests {
    use super::*;
    use flate2::{write::ZlibEncoder, Compression};
    use std::io::Write;

    #[test]
    fn test_decompress_section_with_limits_enforces_limit() {
        let input = vec![b'a'; 1024 * 1024]; // 1MB
        let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
        enc.write_all(&input).unwrap();
        let compressed = enc.finish().unwrap();

        let err = decompress_section_with_limits(&compressed, 1024).unwrap_err();
        match err {
            HwpError::SizeLimitExceeded(_) => {}
            other => panic!("expected SizeLimitExceeded, got: {:?}", other),
        }
    }

    #[test]
    fn test_parse_section_records_uncompressed_size_limit() {
        let data = vec![0u8; 4096];
        let limits = SectionLimits { max_decompressed_bytes: 1024, max_records: 10 };
        let err = parse_section_records_with_options(&data, false, limits).unwrap_err();
        match err {
            HwpError::SizeLimitExceeded(_) => {}
            other => panic!("expected SizeLimitExceeded, got: {:?}", other),
        }
    }
}
