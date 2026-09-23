# Release validation

Local validation performed September 23, 2026 on macOS 26.6.2 / Apple Silicon, with Rust 1.88.0, RISC Zero SDK 3.0.6, and the RISC Zero Rust guest toolchain 1.88.0. These are observed results for this release, not cross-platform performance promises or an independent security audit.

## Rules and CLI

- 19 deterministic rule tests passed: exact thresholds, time-window edges, survey denominators, duplicate records, consent flags, identity mismatches, overflow, schema/size limits, commitments, and disclosure checks.
- Four lightweight CLI tests passed: outcomes, safe expectation creation, malformed/missing/oversized inputs, and refusal to pretend the lightweight build can produce proofs.
- Production proof-feature unit/CLI tests passed, including explicit development-receipt rejection.
- Formatting and lightweight Clippy checks passed without warnings.
- Both README diagrams were rendered and visually inspected.

## Real proofs

`scripts/proof_roundtrip.py` passed **36 subprocess checks using five real composite receipts**. Every example was previewed, proved locally, and independently verified in a new process. Preview, proved, and verified journals agreed exactly.

| Example | Verified result | Observed proving time |
| --- | --- | --- |
| Affirmative market | met | 21 seconds |
| Negative market | not_met | 22 seconds |
| Unresolved market | indeterminate | 20 seconds |
| Social views | met; 4 posts, 45,000 views | 24 seconds |
| Participant survey | met; 4 of 5 respondents qualify | 62 seconds |

Timings include local process startup and receipt writing, exclude the initial build, and were measured with other build activity on the same machine. The JSON receipts were approximately 659–691 KB. They are generated artifacts, not committed source fixtures.

The verifier rejected a modified journal, a modified proof seal, a changed policy/run ID, a changed evidence digest, an expectation for a different metric, and a truncated receipt. The CLI also rejected ambient development mode cleanly, refused to overwrite a receipt, and refused duplicate-respondent evidence before proving. `--require-met` returned exit code 2 for valid negative and unresolved outcomes.

An additional receipt test accepted the real market receipt under the compiled guest identity and rejected it under an altered program identity.

Tested guest image ID:

```text
5a3e8df2d156262a75ede3be90c9e2df3d11828a54d82af4e39468da60ff6fb3
```

An image ID is a program identity, not an assertion that the source data is authentic. Rebuilding with different tools or dependencies may produce a different ID; see the [development guide](DEVELOPMENT.md).

## Remaining limits

No live source connector, cryptographic source attestation, sentiment classifier, chain submission, payout system, replay database, or hosted CI is included. The proof authenticates the calculation over supplied evidence. The [trust model](TRUST.md) states the assumptions that remain outside that proof.
