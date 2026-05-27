# Release Checklist

This maintainer checklist is for the first public release and any material public update. It is a release gate, not a claim that Evidence Packet verifies evidence or certifies truth.

## Data Boundary

- [ ] Examples are synthetic only.
- [ ] No private, personal, client, recruiter, employer, school, family, or controlled Unit27 material is present.
- [ ] No real `.env`, token, credential, key, database, cache, or scratch artifacts are included.
- [ ] Risky-artifact examples remain in tests only.

## Claim Boundary

- [ ] README states that the tool is a local artifact-packaging aid.
- [ ] README states that hashes show file identity, not truth.
- [ ] README does not claim verification, certification, notarization, fraud prevention, compliance review, legal review, medical safety review, or production readiness.
- [ ] Generated examples include the boundary note.

## Technical Gate

- [ ] `cargo fmt -- --check`
- [ ] `cargo test`
- [ ] `cargo clippy -- -D warnings`
- [ ] Synthetic demo regenerated:

```bash
cargo run -- create examples/sample_artifacts \
  --claim "The local CLI generated an evidence packet from synthetic artifacts." \
  --scope "one local demo using synthetic files" \
  --limitations "does not establish production readiness" \
  --limitations "does not verify the truth of the claim" \
  --output examples/generated
```

## Release Gate

- [ ] GitHub Actions CI is present.
- [ ] MIT license is present.
- [ ] Public file scan has no private or risky material beyond boundary language.
- [ ] Joshua explicitly approves first remote repo creation and push.
- [ ] No public push, post, deploy, or announcement happens before approval.
