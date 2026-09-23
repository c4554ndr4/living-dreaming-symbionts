# Design history

The July 2025 prototype explored AI companions whose goals could be connected to externally observable outcomes. Its three concrete examples were audience reach, a project-completion market, and an opt-in participant survey. The central question was architectural: how could a person inspect an agent's success claim without relying entirely on the agent's narrative?

The original program demonstrated RISC Zero execution over hardcoded examples. It did not implement the proposed live collection, source authentication, blockchain submission, or incentive distribution. The raw design conversation mixed research ideas with unfinished implementation suggestions; this public edition replaces that transcript with curated documentation and synthetic examples. Original development history is retained privately.

The September 2026 release completes the local verification layer. It shares one set of rules between preview and proof execution; derives aggregates from individual records; distinguishes affirmative, negative, and pending outcomes; binds results to policy and evidence; and saves receipts that a separate process can verify. It also replaces floating proof dependencies with a patched, pinned SDK, retains lockfiles, and rejects simulated receipts.

## Why keep the proof program small?

The most useful boundary is between external observation and deterministic verification. Browsing, language-model interpretation, consent collection, and market resolution all have their own trust assumptions. Putting their outputs into a proof does not remove those assumptions. A small proof program makes the exact verified claim easier to inspect and test.

## Next research questions

1. **Authenticity and completeness:** how does a verifier know who produced the records and whether inconvenient records were omitted?
2. **Metric quality:** does optimizing the chosen score actually help the people the agent serves? How should disagreement, withdrawals, and appeals work?
3. **Composition:** how should multiple metrics, observation periods, and contradictory outcomes combine into a decision?
4. **Operational independence:** can multiple parties reproduce the guest build, retain expectations independently, and audit the collection path?

Live connectors, signed attestations, TLSNotary, chain-state proofs, scheduling, and reward distribution remain future integrations. They are not implied by a successfully verified receipt in this release.
