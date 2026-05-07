#!/usr/bin/env python3
import argparse
import json
import urllib.request
from pathlib import Path


def load_entries(dataset_path):
    with Path(dataset_path).open() as f:
        data = json.load(f)
    return data["entries"]


def main():
    parser = argparse.ArgumentParser(description="Smoke test /ready and two official /fraud-score payloads")
    parser.add_argument("--base-url", default="http://127.0.0.1:9999", help="Base URL for the stack")
    parser.add_argument("--dataset-path", required=True, help="Path to official test-data.json")
    parser.add_argument("--timeout", type=float, default=5.0, help="Per-request timeout in seconds")
    args = parser.parse_args()

    entries = load_entries(args.dataset_path)
    ready_url = f"{args.base_url.rstrip('/')}/ready"
    score_url = f"{args.base_url.rstrip('/')}/fraud-score"

    with urllib.request.urlopen(ready_url, timeout=args.timeout) as response:
        ready_status = response.status
        ready_body = response.read().decode(errors="replace")

    checks = []
    headers = {"Content-Type": "application/json"}
    for idx in [0, 1]:
        payload = json.dumps(entries[idx]["request"]).encode()
        request = urllib.request.Request(score_url, data=payload, headers=headers, method="POST")
        with urllib.request.urlopen(request, timeout=args.timeout) as response:
            body = response.read().decode()
            checks.append(
                {
                    "entry_index": idx,
                    "status": response.status,
                    "body": json.loads(body),
                    "expected_approved": entries[idx]["expected_approved"],
                    "expected_fraud_score": entries[idx]["expected_fraud_score"],
                }
            )

    print(
        json.dumps(
            {
                "ready": {"status": ready_status, "body": ready_body},
                "checks": checks,
            },
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
