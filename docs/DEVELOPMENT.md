# Development and proof walkthrough

## Fast path

From the repository root, install the Rust toolchain specified in `rust-toolchain.toml` through [rustup](https://rustup.rs/), then run:

```sh
cargo test --locked -p symbiont-core -p symbiont
cargo run --locked -p symbiont -- evaluate examples/social-views.json
cargo run --locked -p symbiont -- evaluate examples/market-no.json
cargo run --locked -p symbiont -- evaluate examples/participant-survey.json
```

This path does not compile the proof engine or require the RISC Zero guest toolchain. Results are previews, clearly labeled on stderr. The CLI writes JSON results to stdout, keeping machine-readable output separate from status messages.

## Generate and verify a real proof

Install RISC Zero's toolchain manager using its [official installation instructions](https://dev.risczero.com/api/zkvm/install). This repository pins the SDK to **3.0.6** and uses the RISC Zero Rust guest toolchain **1.88.0**. The SDK is newer than the original prototype and includes the fix for [the guest input memory-safety advisory](https://github.com/risc0/risc0/security/advisories/GHSA-jqq4-c7wq-36h7).

```sh
rzup install rust 1.88.0
RISC0_BUILD_LOCKED=1 cargo build --locked --release -p symbiont --features zk
mkdir -p artifacts

target/release/symbiont prepare examples/market-yes.json artifacts/expected.json
target/release/symbiont prove examples/market-yes.json artifacts/receipt.json
target/release/symbiont verify artifacts/receipt.json --expect artifacts/expected.json --require-met
target/release/symbiont image-id
```

Review the expectation before relying on it. In a multi-party workflow, the verifier retains its own expectation and receives only the receipt from the prover. Repeat the commands with fresh output paths: existing files are never overwritten. The market example is the smallest proof. Social and survey examples use the same path.

The `zk` feature builds the guest and local proof engine. Initial compilation is substantially larger than the preview build; proof generation also uses considerably more CPU and memory than evaluating the rules. No cloud account or API key is required. See [release validation](VALIDATION.md) for the platform and cases actually exercised.

The executable refuses fake receipts. Both proving and verification explicitly disable development mode, and the SDK is built with `disable-dev-mode`. There is no command that silently generates a development receipt.

If `RISC0_DEV_MODE` is enabled in your shell, the CLI exits with an explanatory error before entering the SDK. Unset it to run the real proof workflow.

## Protocol

Each input contains a `policy` and `evidence`. See the five files in [examples](../examples) for complete schemas.

- Schema version is `1`; unknown fields and variants are rejected.
- `run_id` is a verifier-chosen evaluation identifier, not a clock or an automatic replay database.
- Strings must be 1–256 bytes, with no surrounding whitespace or control characters.
- Input JSON is limited to 1 MiB; lists to 1,000 posts/respondents; surveys to 32 questions. The prover limits execution to 32 million cycles. Large valid inputs may exceed this execution budget.
- Social timestamps are Unix seconds. The window includes its start and excludes its end. Views are unsigned integers, summed with overflow checking. `views_must_exceed` is a strict comparison.
- Survey thresholds use basis points: `8000` means 80%. All submitted respondents must consent and answer every question. IDs must be unique. The denominator is the submitted consenting cohort, whose real-world completeness is not authenticated. A minimum sample that is not reached produces `indeterminate`.
- Market resolutions are `yes`, `no`, `unresolved`, and `cancelled`. Market ID and question must match the policy exactly; include the provider or chain namespace in the ID.
- SHA-256 commitments hash domain-separated, typed compact JSON. The domains are `symbiont/policy/v1` and `symbiont/evidence/v1`, followed by one zero byte and the serialized value. Whitespace and object field order in the source JSON do not affect the commitment; array order, string contents, and values do. This is this protocol's encoding, not a general-purpose JSON canonicalization standard.
- A receipt is RISC Zero's JSON-serialized `Receipt`, capped at 64 MiB on input. Its journal is a JSON `Journal`. The verifier uses the compiled guest image ID, never an ID supplied in the receipt file.
- `evaluate` / `prepare` / `prove` return 0 on a valid calculation, including an unmet goal. `verify` returns 0 for a valid matching receipt; `--require-met` returns 2 for a valid non-met result. Input, I/O, and verification failures return 1.

Commit both workspace and guest lockfiles when updating dependencies. Guest and host share `symbiont-core`. Changes to the guest, shared rules, dependencies, or compiler may change the image ID; recompile verifier and prover together and review the new identity.

## Checks

```sh
cargo fmt --all --check
cargo fmt --manifest-path methods/guest/Cargo.toml --check
cargo clippy --locked -p symbiont-core -p symbiont -- -D warnings
cargo test --locked -p symbiont-core -p symbiont
RISC0_BUILD_LOCKED=1 cargo test --locked --release -p symbiont --features zk
python3 scripts/proof_roundtrip.py target/release/symbiont
SYMBIONT_TEST_RECEIPT=artifacts/receipt.json RISC0_BUILD_LOCKED=1 cargo test --locked --release -p symbiont --features zk real_receipt_rejects_a_different_program -- --ignored
```

The Python check generates real local receipts for every example, independently verifies them, exercises the negative/pending exit codes, and rejects modified journals, seals, policies, and evidence expectations. It takes longer than the unit suite. The last command checks the saved walkthrough receipt against the correct and an incorrect program identity. No hosted CI service is configured in this initial release.

Mermaid diagram sources are committed alongside their rendered SVGs. Regenerate them with Mermaid CLI if changing the architecture illustrations.
