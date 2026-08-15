# Semver Policy

This crate uses semantic versioning for the published Rust API. The authoritative
package version is the `version` field in `Cargo.toml`.

## Stable API compatibility

- Patch releases fix bugs, improve documentation, and make compatible internal
  changes.
- Minor releases may add new public APIs while preserving compatibility with
  existing supported APIs.
- Major releases may remove or change public APIs, public behavior, feature
  names, or documented protocol expectations.

## Technical preview and pre-1.0 posture

The SDK is currently documented as a technical preview. Before the maintainers
declare a stable SDK contract, breaking changes may be accepted when they are
needed to match the Copilot CLI agent runtime or to correct an API design issue.
Those changes must be documented in `CHANGELOG.md`.

For any pre-1.0 release line, minor version bumps may include breaking changes
and patch releases should remain compatible within that minor line unless a
critical runtime compatibility fix requires otherwise.

## 1.0 transition

The first release that declares a stable SDK contract will document that
transition in `CHANGELOG.md` and in the README. After that transition, breaking
changes to supported public APIs require a major version bump, except for APIs
that are explicitly documented as unstable, experimental, generated, or tied to
preview Copilot CLI runtime behavior.

## MSRV

Rust compatibility is governed separately by
[`docs/msrv-policy.md`](msrv-policy.md).
