# Benchmarks (WIP)

Goal: publish *reproducible* metrics.

## Metrics
- Parse success rate (by corpus)
- Time per document (ms)
- Memory peak (RSS)
- Markdown fidelity score (golden fixtures diff)
- Encrypted/distribution file detection accuracy

## How to run
```bash
cargo test -p hwp-core -- --nocapture
# TODO: add a `bench` crate / criterion benches
```
