# Trust and privacy model

## What a verified receipt means

For the program compiled by this verifier, RISC Zero verifies successful execution and the integrity of the public journal. The program validates the input, evaluates its metric, and commits the result together with SHA-256 fingerprints of the policy and evidence. The verifier compares both fingerprints with an independently retained expectation.

Trust the verifier's source, build, program identity, cryptographic dependencies, and chosen expectation. Do not accept an expectation or replacement verifier merely because the prover attached it to a receipt. An altered verifier can approve anything. Lockfiles constrain dependencies; they are not a claim of independently reproduced binary builds or a security audit.

## What it does not mean

- The account's posts are genuine or the list is complete.
- A market oracle or website actually reported the supplied resolution.
- Respondent IDs correspond to unique people, people truly consented, or all eligible participants were sampled.
- Positive-answer labels faithfully reflect a person's words. There is no sentiment model in the proof.
- A timestamp is current, the data was collected at a claimed time, or the agent caused the outcome.
- Meeting the metric constitutes AI alignment, user wellbeing, or a safe basis for automatic rewards.

These require collection protocols, authenticated sources, consent procedures, and human judgment beyond this release. The source-authenticity regression test deliberately demonstrates that internally consistent invented evidence can pass the rules.

## Replay and expectations

The policy's `run_id` is committed into the proof. Use a new verifier-chosen value for each evaluation and retain the expected policy and evidence digest through an independent channel. The CLI does not maintain a used-ID database, enforce expiration, or know whether an application has already accepted a receipt. Consumers must implement those checks when repetition matters.

`prepare` computes an expectation from a locally reviewed input. It is convenient when a verifier has the evidence, or when parties agree on a digest in advance. It is not a way to authenticate data sent by an untrusted prover. Receipt verification itself needs only the expectation and receipt, not the individual records.

## Disclosure

The journal exposes the outcome, aggregate counts or market resolution, and policy/evidence digests. It omits individual records. A policy, outcome, or small aggregate may still identify someone. Unsalted evidence digests permit guessing attacks against low-entropy inputs; this protocol does not claim strong anonymity or confidentiality. Share receipts and expectation files deliberately.

The CLI has no source connector, telemetry, or remote proving path. Proving is explicitly local. Dependency downloads and initial proof-engine setup may access package and upstream distribution servers. New output files are created with owner-only permissions on Unix and are never overwritten. Users remain responsible for the permissions and storage of their input files.

## Invalid evidence versus an unmet goal

Malformed or mismatched evidence is rejected and produces no receipt. Well-formed evidence can produce `not_met` or `indeterminate`; these can have perfectly valid proofs. `verify --require-met` adds the application's success requirement after cryptographic verification, returning exit code 2 for either non-met outcome. A verification or input error returns exit code 1.

The project has automated adversarial cases and a locally exercised proof path. It has not undergone an independent security audit and is not a deployed oracle, payout system, or production alignment mechanism.
