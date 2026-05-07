# rinha-de-backend-2026-fabio-rust

Rust implementation for Rinha de Backend 2026.

Status: in progress.

## Benchmark discipline

Use optimized release builds for any meaningful benchmark:

```bash
cargo build --release
```

Never trust debug measurements for latency decisions.

## Local benchmark helpers

Baseline and benchmark notes live in:

- `docs/benchmark-baseline.md`

Reusable local helpers:

- `python3 scripts/smoke_stack.py --dataset-path ~/Repos/rinha-de-backend-2026/test/test-data.json`
- `python3 scripts/benchmark_direct.py --dataset-path ~/Repos/rinha-de-backend-2026/test/test-data.json --base-url http://127.0.0.1:9999`
- `python3 scripts/benchmark_stack.py --dataset-path ~/Repos/rinha-de-backend-2026/test/test-data.json --base-url http://127.0.0.1:9999`
