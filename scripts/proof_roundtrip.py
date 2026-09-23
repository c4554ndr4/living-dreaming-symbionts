#!/usr/bin/env python3
"""Exercise real proof/verify subprocesses and adversarial receipt cases.

Only synthetic repository examples are used. Requires the release binary built
with --features zk. No Python packages, credentials, or remote prover are used.
"""
import argparse
import copy
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--output-dir", type=Path, help="Retain synthetic receipts and the test report in a new directory")
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    temporary = None
    if args.output_dir:
        args.output_dir.mkdir(parents=True, exist_ok=False)
        directory = args.output_dir.resolve()
    else:
        temporary = tempfile.TemporaryDirectory(prefix="symbiont-proof-test-")
        directory = Path(temporary.name)
    environment = os.environ.copy()
    environment.pop("RISC0_DEV_MODE", None)
    checks = []

    def run(label, *arguments, code=0, env=None):
        start = time.monotonic()
        result = subprocess.run([str(binary), *map(str, arguments)], capture_output=True,
                                text=True, timeout=1800, env=env or environment)
        if result.returncode != code:
            raise AssertionError(f"{label}: expected exit {code}, got {result.returncode}\n{result.stderr}")
        if "panicked" in result.stderr:
            raise AssertionError(f"{label}: unexpected panic\n{result.stderr}")
        checks.append({"check": label, "exit_code": code, "seconds": round(time.monotonic() - start, 3)})
        print(f"PASS {label}", flush=True)
        return result

    def save(name, value):
        path = directory / name
        path.write_text(json.dumps(value), encoding="utf-8")
        return path

    examples = [("market-yes", "met"), ("market-no", "not_met"),
                ("market-unresolved", "indeterminate"), ("social-views", "met"),
                ("participant-survey", "met")]
    for name, outcome in examples:
        source = ROOT / "examples" / f"{name}.json"
        expected = directory / f"{name}.expected.json"
        receipt = directory / f"{name}.receipt.json"
        run(f"prepare {name}", "prepare", source, expected)
        preview = json.loads(run(f"preview {name}", "evaluate", source).stdout)
        proved = json.loads(run(f"real proof {name}", "prove", source, receipt).stdout)
        verified = json.loads(run(f"independent verification {name}", "verify", receipt, "--expect", expected).stdout)
        assert preview == proved == verified and verified["outcome"] == outcome
        artifact = json.loads(receipt.read_text())
        assert "Fake" not in artifact["inner"], "prover emitted a development receipt"
        run(f"require-met {name}", "verify", receipt, "--expect", expected,
            "--require-met", code=0 if outcome == "met" else 2)

    receipt_path = directory / "market-yes.receipt.json"
    expected_path = directory / "market-yes.expected.json"
    original = json.loads(receipt_path.read_text())
    expectation = json.loads(expected_path.read_text())

    changed = copy.deepcopy(original)
    journal = json.loads(bytes(changed["journal"]["bytes"]))
    journal["outcome"] = "not_met"
    changed["journal"]["bytes"] = list(json.dumps(journal, separators=(",", ":")).encode())
    run("reject modified journal", "verify", save("tampered-journal.json", changed), "--expect", expected_path, code=1)

    changed = copy.deepcopy(original)
    # Default local proving emits composite receipts with at least one segment.
    seal = changed["inner"]["Composite"]["segments"][0]["seal"]
    seal[len(seal) // 2] ^= 1
    run("reject modified proof seal", "verify", save("tampered-seal.json", changed), "--expect", expected_path, code=1)

    changed = copy.deepcopy(expectation)
    changed["policy"]["run_id"] += "-different-run"
    run("reject different policy/run", "verify", receipt_path, "--expect", save("wrong-policy.json", changed), code=1)
    changed = copy.deepcopy(expectation)
    changed["evidence_sha256"] = "0" * 64
    run("reject different evidence", "verify", receipt_path, "--expect", save("wrong-evidence.json", changed), code=1)
    run("reject different metric", "verify", receipt_path, "--expect", directory / "participant-survey.expected.json", code=1)

    truncated = directory / "truncated.json"
    truncated.write_bytes(receipt_path.read_bytes()[:100])
    run("reject truncated receipt", "verify", truncated, "--expect", expected_path, code=1)

    # A malicious consumer cannot get a positive exit simply by enabling SDK development mode.
    development = {**environment, "RISC0_DEV_MODE": "1"}
    run("reject ambient development mode", "verify", receipt_path, "--expect", expected_path, code=1, env=development)
    run("refuse development proving", "prove", ROOT / "examples/market-yes.json", directory / "must-not-exist.json", code=1, env=development)
    assert not (directory / "must-not-exist.json").exists()

    before = receipt_path.read_bytes()
    run("refuse receipt overwrite", "prove", ROOT / "examples/market-yes.json", receipt_path, code=1)
    assert before == receipt_path.read_bytes()

    invalid = json.loads((ROOT / "examples/participant-survey.json").read_text())
    invalid["evidence"]["respondents"][1]["id"] = invalid["evidence"]["respondents"][0]["id"]
    run("refuse invalid evidence before proving", "prove", save("duplicate-respondent.json", invalid), directory / "invalid-receipt.json", code=1)
    assert not (directory / "invalid-receipt.json").exists()

    image = run("guest image identity", "image-id").stdout.strip()
    report = {"sdk": "3.0.6", "image_id": image, "real_receipts": len(examples), "checks": checks}
    save("report.json", report)
    print(f"Passed {len(checks)} checks with {len(examples)} real receipts. Guest image: {image}")
    if temporary:
        temporary.cleanup()
    else:
        print(f"Synthetic receipts and report: {directory}")


if __name__ == "__main__":
    main()
