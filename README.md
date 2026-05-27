# Evidence Packet

[![CI](https://github.com/unit27research/evidence-packet/actions/workflows/ci.yml/badge.svg)](https://github.com/unit27research/evidence-packet/actions/workflows/ci.yml)

Artifact does not equal proof.

Evidence Packet is a small local Rust instrument for packaging evidence artifacts with hashes, declared scope, limitations, and a clear boundary note.

It records what was supplied. It does not prove that the claim is true.

The category is evidence-packet review: artifact does not equal proof, boundary before scale, and review before release.

## Release Status

`SOURCE_STATUS: PUBLIC_PACKAGE`
`ACCESS_STATUS: CLEARED_FOR_EXTERNAL_USE`
`UNIT27_POSITION: ADJACENT_EVIDENCE_BOUNDARY_UTILITY`

This repository is a Unit27 public utility: visible, inspectable, and intended for orientation, testing, and practical use. Controlled protocol materials remain outside this source package.

It answers one narrow question:

> What artifacts were supplied, and what boundary was declared around them?

## Failure Mode

Evidence packets help with a common proofwashing problem: a screenshot, local output, synthetic demo, or one run may be real, but the artifact alone does not establish the broader claim someone may want to make.

The packet keeps the artifact record and the evidence boundary in the same review surface:

- claim being supported
- supported scope
- known limitations
- artifact paths
- file sizes
- SHA-256 hashes
- risk flags for obvious private, scratch, cache, or generated-local files

## What Evidence Packet Does

Evidence Packet reads a local artifact directory and writes a bounded packet:

- `evidence_packet.json`
- `EVIDENCE_PACKET.md`

The packet includes:

- declared claim
- supported scope
- limitations
- boundary note
- artifact list
- SHA-256 hashes
- risk flags

By default, it refuses obvious risky files such as `.env`, `.env.local`, key files, token files, credential files, private files, local databases, cache files, scratch files, `.DS_Store`, and `.pyc` files.

Use `--allow-risky` only for synthetic or intentionally reviewed material. When `--allow-risky` is used, `--risk-reviewed` is required so the packet records why the risky artifact was allowed.

## What It Does Not Do

Evidence Packet is not a verifier, fact-checker, fraud detector, certification system, compliance system, legal reviewer, medical safety tool, or truth oracle.

It does not inspect external sources, validate screenshots, audit code, certify evidence, or decide what should be published.

It is a local artifact-packaging aid. The output is a review surface for human judgment, not final authority.

## Where It Fits

Evidence Packet sits beside the Proofwashing Suite as an adjacent evidence-boundary utility.

- Proofwashing Suite asks: "Which claim-review instrument should be used?"
- Humility Engine asks: "Does this claim outrun the evidence?"
- Evidence Floor asks: "Does this claim meet the minimum evidence required for this claim class?"
- Evidence Packet asks: "What artifact record is actually being supplied, and what boundary is declared around it?"

## Who It Is For

- builders preserving proof before claim
- researchers and operators packaging artifact records before release
- teams keeping demo evidence separate from broader readiness claims
- anyone who wants a local manifest before citing artifacts in public copy

## Quick Demo

Build and run the synthetic example:

```bash
cargo run -- create examples/sample_artifacts \
  --claim "The local CLI generated an evidence packet from synthetic artifacts." \
  --scope "one local demo using synthetic files" \
  --limitations "does not establish production readiness" \
  --limitations "does not verify the truth of the claim" \
  --output examples/generated
```

Open:

- `examples/generated/evidence_packet.json`
- `examples/generated/EVIDENCE_PACKET.md`

## Example Boundary Note

Every packet includes this boundary:

> This packet records supplied artifacts and declared boundaries. It does not prove that the declared claim is true. It does not verify truth, and it does not certify safety, compliance, or readiness.

## Current Limits

- Hashes show file identity, not truth.
- Timestamps use local creation time for the packet, not independent notarization.
- Risk-file detection is pattern-based and incomplete.
- `--allow-risky` records a review note; it does not make risky material safe.
- The tool does not inspect artifact contents beyond hashing bytes.
- The tool does not upload, publish, sign, notarize, or externally corroborate packets.

## Reliability

CI verifies formatting, unit tests, Clippy warnings, and the synthetic demo command before changes are considered ready.

## Verify

```bash
cargo fmt -- --check
cargo test
cargo clippy -- -D warnings
cargo run -- create examples/sample_artifacts \
  --claim "The local CLI generated an evidence packet from synthetic artifacts." \
  --scope "one local demo using synthetic files" \
  --limitations "does not establish production readiness" \
  --limitations "does not verify the truth of the claim" \
  --output examples/generated
```

## License

MIT
