# Benchmark baseline

## Objective

This document freezes the current local baseline for `~/Repos/rinha-de-backend-2026-fabio-rust` so future optimizations can be compared against a reproducible reference.

## Environment

- Machine used for local measurements: this macOS workstation
- API resources: mmap baseline at `/tmp/rinha-mmap-baseline`
- Official dataset for smoke/bench payloads: `~/Repos/rinha-de-backend-2026/test/test-data.json`
- Topology for stack benchmark:
  - nginx local on port `9999`
  - API instance 1 on `127.0.0.1:10001`
  - API instance 2 on `127.0.0.1:10002`
- App binary used for realistic measurements:
  - `~/Repos/rinha-de-backend-2026-fabio-rust/target/release/rinha-de-backend-2026-fabio-rust`

## Correctness baseline

Official dataset evaluation baseline:

- total: `54100`
- correct: `52876`
- accuracy: `0.977375`
- precision_fraud: `0.975471`
- recall_fraud: `0.973605`
- false_positive: `589`
- false_negative: `635`
- avg_score_error: `0.011309`

HTTP smoke baseline:

- `GET /ready` -> `200`
- official fraud sample -> `{"approved":false,"fraud_score":1.0}`
- official legit sample -> `{"approved":true,"fraud_score":0.0}`

## Direct release baseline

Single app process, release build, warmed.

### Concurrency 1
- count: `600`
- rps: `2038.90`
- avg: `0.477 ms`
- p50: `0.416 ms`
- p95: `0.968 ms`
- p99: `1.548 ms`
- max: `1.818 ms`

### Concurrency 8
- run 1:
  - rps: `5020.99`
  - avg: `1.558 ms`
  - p99: `6.317 ms`
- run 2:
  - rps: `5161.70`
  - avg: `1.512 ms`
  - p99: `3.112 ms`
- run 3:
  - rps: `5249.03`
  - avg: `1.491 ms`
  - p99: `2.862 ms`

### Concurrency 16
- count: `600`
- rps: `5254.76`
- avg: `2.936 ms`
- p50: `2.875 ms`
- p95: `4.758 ms`
- p99: `5.810 ms`
- max: `6.327 ms`

### Concurrency 32
- count: `600`
- rps: `5219.10`
- avg: `5.581 ms`
- p50: `5.368 ms`
- p95: `9.480 ms`
- p99: `10.556 ms`
- max: `13.199 ms`

## nginx + 2 instances baseline

Real nginx locally installed through Homebrew, 2 release API instances behind round-robin.

Warmup:
- concurrency: `16`
- count: `300`
- rps: `4964.56`
- avg: `2.994 ms`
- p95: `4.846 ms`
- p99: `5.665 ms`
- max: `5.883 ms`

### Concurrency 1
- count: `600`
- rps: `2027.47`
- avg: `0.479 ms`
- p50: `0.464 ms`
- p95: `0.951 ms`
- p99: `1.694 ms`
- max: `1.772 ms`

### Concurrency 8
- run 1:
  - count: `600`
  - rps: `5016.80`
  - avg: `1.557 ms`
  - p95: `2.487 ms`
  - p99: `6.184 ms`
  - max: `7.447 ms`
- run 2:
  - count: `600`
  - rps: `5250.59`
  - avg: `1.490 ms`
  - p95: `2.440 ms`
  - p99: `3.180 ms`
  - max: `3.497 ms`
- run 3:
  - count: `600`
  - rps: `5130.81`
  - avg: `1.525 ms`
  - p95: `2.489 ms`
  - p99: `3.000 ms`
  - max: `3.663 ms`

### Concurrency 16
- count: `600`
- rps: `5053.07`
- avg: `3.050 ms`
- p50: `2.973 ms`
- p95: `4.903 ms`
- p99: `5.547 ms`
- max: `6.608 ms`

### Concurrency 32
- count: `600`
- rps: `5042.34`
- avg: `5.811 ms`
- p50: `5.702 ms`
- p95: `9.828 ms`
- p99: `11.788 ms`
- max: `15.273 ms`

## Notes

- The competition scoring saturates latency points at `p99 <= 1ms`, so current work should focus on reducing tail latency under concurrency.
- Future changes should be measured in release mode with warmup and compared against this baseline before being accepted.
- Approximate search improvements must always be paired with official dataset validation to avoid silent regressions in fraud detection quality.
