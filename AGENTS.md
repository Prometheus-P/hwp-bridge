# Repository Guidelines

## Project Structure & Module Organization
HwpBridge is a Rust workspace (see `Cargo.toml`) with domain crates under `crates/`. `hwp-core` owns parsers and converters, `hwp-types` defines shared structs/enums, `hwp-cli` exposes conversion commands, `hwp-mcp` adds MCP tooling, and `hwp-wasm` compiles browser bindings. Shared docs/specs live in `docs/` (study `docs/specs/ARCHITECTURE.md` before changing formats). Deployment manifests are under `deploy/`, static assets/public API mocks under `public/`, and example corpora + fixtures sit in `hwpers/`. Put new integration tests in the owning crate's `tests/` dir; cross-cutting E2E cases go to `tests/e2e/`.

## Build, Test, and Development Commands
- `cargo build --workspace` – default dev build for every crate.
- `cargo build --workspace --release` – optimized build used by `deploy/`.
- `cargo run -p hwp-cli -- convert docs/samples/foo.hwp -o out.html` – exercise CLI pipeline locally.
- `cargo watch -x "test --workspace"` – rapid TDD loop.
- `cargo test --workspace --all-features` – CI-equivalent suite; use `cargo test -p hwp-core parser::header` when iterating.
- `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings` – formatting + lint gate before commits.
- `cargo audit` – dependency scanning prior to release work.

## Coding Style & Naming Conventions
Follow `rustfmt` defaults (4-space indent, max 100 columns). Modules/functions stay `snake_case`, public types/traits use `PascalCase`, constants remain `SCREAMING_SNAKE_CASE`. Keep files under ~400 lines and functions under ~20 lines as noted in `CONTRIBUTING.md`. Document public APIs with Rustdoc blocks that explain arguments, errors, and sample usage (see `crates/hwp-core/src/parser/header.rs`). Prefer explicit error enums (`HwpError`) and avoid panics in parsing paths.

## Testing Guidelines
Unit tests live beside source files inside `#[cfg(test)]` modules; target ≥80% coverage and mirror scenarios in `docs/guides/TEST_STRATEGY.md`. Integration suites reside in `crates/*/tests/` and operate on fixture `.hwp` files stored under `tests/fixtures/`. End-to-end flows (CLI, web, MCP) belong in `tests/e2e/` and should assert observable behavior rather than internal structs. Name tests descriptively (`test_should_parse_valid_header`) and keep failure messages actionable. Run `cargo clippy` and the workspace test suite before opening a PR; append `--nocapture` when debugging parser differences.

## Commit & Pull Request Guidelines
Commits follow `type(scope): summary (#issue)` as seen in `git log` (`chore(ci): …`). Squash noisy WIP commits locally and keep messages imperative ("add table parser"). Every PR must describe the change, link the driving issue/spec, list manual test commands, and attach screenshots/log snippets for CLI or API changes. Checkboxes for `cargo fmt`, `cargo clippy`, and `cargo test --workspace` should be completed before requesting review.
