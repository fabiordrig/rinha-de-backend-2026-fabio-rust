#!/usr/bin/env python3
import argparse
import json
import time
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path


def percentile(sorted_vals, p):
    if not sorted_vals:
        return 0.0
    if len(sorted_vals) == 1:
        return sorted_vals[0]
    rank = (len(sorted_vals) - 1) * p
    lo = int(rank)
    hi = min(lo + 1, len(sorted_vals) - 1)
    frac = rank - lo
    return sorted_vals[lo] * (1 - frac) + sorted_vals[hi] * frac


def load_requests(dataset_path, limit):
    with Path(dataset_path).open() as f:
        data = json.load(f)
    entries = data["entries"][:limit]
    return [json.dumps(entry["request"]).encode() for entry in entries]


def run_benchmark(url, requests, concurrency, total, timeout):
    headers = {"Content-Type": "application/json"}
    latencies = []
    statuses = {}
    started = time.perf_counter()

    def one_call(i):
        payload = requests[i % len(requests)]
        req = urllib.request.Request(url, data=payload, headers=headers, method="POST")
        t0 = time.perf_counter()
        with urllib.request.urlopen(req, timeout=timeout) as response:
            response.read()
            dt_ms = (time.perf_counter() - t0) * 1000
            return dt_ms, response.status

    with ThreadPoolExecutor(max_workers=concurrency) as executor:
        futures = [executor.submit(one_call, i) for i in range(total)]
        for future in as_completed(futures):
            dt_ms, status = future.result()
            latencies.append(dt_ms)
            statuses[str(status)] = statuses.get(str(status), 0) + 1

    elapsed = time.perf_counter() - started
    latencies.sort()
    return {
        "count": len(latencies),
        "rps": round(len(latencies) / elapsed, 2),
        "avg_ms": round(sum(latencies) / len(latencies), 3),
        "p50_ms": round(percentile(latencies, 0.50), 3),
        "p95_ms": round(percentile(latencies, 0.95), 3),
        "p99_ms": round(percentile(latencies, 0.99), 3),
        "max_ms": round(max(latencies), 3),
        "statuses": statuses,
    }


def main():
    parser = argparse.ArgumentParser(description="Run concurrent benchmark against /fraud-score")
    parser.add_argument("--base-url", default="http://127.0.0.1:9999", help="Base URL for the stack")
    parser.add_argument("--dataset-path", required=True, help="Path to official test-data.json")
    parser.add_argument("--dataset-limit", type=int, default=50, help="Number of dataset entries to cycle through")
    parser.add_argument("--concurrency", type=int, default=16, help="Number of concurrent workers")
    parser.add_argument("--total", type=int, default=600, help="Total requests to execute")
    parser.add_argument("--timeout", type=float, default=10.0, help="Per-request timeout in seconds")
    args = parser.parse_args()

    requests = load_requests(args.dataset_path, args.dataset_limit)
    result = run_benchmark(
        f"{args.base_url.rstrip('/')}/fraud-score",
        requests,
        args.concurrency,
        args.total,
        args.timeout,
    )
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
