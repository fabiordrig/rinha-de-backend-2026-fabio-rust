#!/usr/bin/env python3
import argparse
import json
from benchmark_stack import load_requests, run_benchmark


def main():
    parser = argparse.ArgumentParser(description="Run concurrent benchmark against a single API instance")
    parser.add_argument("--base-url", default="http://127.0.0.1:9999", help="Base URL for the instance")
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
