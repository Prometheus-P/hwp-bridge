# Future (Planned Components)

This folder contains **planned / disabled** components that are **not part of Option A**.

Option A scope (current, supported in this repo snapshot):
- `hwp-core` / `hwp-types`
- `hwp-cli`
- `hwp-wasm`
- `hwp-mcp`

Planned / disabled (moved here):
- `hwp-web` (REST API, Google OAuth/Docs integration)
- Cloud Run / Kubernetes deployment notes for `hwp-web`
- Reverse proxy / monitoring configs assuming `hwp-web`

## Index
- `hwp-web/` — web server specs & deployment notes
- `env/` — web-only environment examples
- `workflows/` — web-only CI/CD workflows (disabled)
- `deploy/` — proxy/monitoring configs assuming web services

> If you later revive `hwp-web`, move the relevant assets back into their original locations
> and re-enable workflows/configs accordingly.
