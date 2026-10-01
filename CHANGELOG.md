# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows the versioning policy documented in
[`docs/semver-policy.md`](docs/semver-policy.md).

## Unreleased

### Added

- Added release policy documentation for changelog, semver, and MSRV handling.
- Added `ModelInfo::new`, `ModelPolicy::new`, and `RawSessionEvent::new` helpers,
  plus defaults for model billing metadata, for construction without literals.

### Changed

- **Breaking:** Public enums in `types` and `events`, and `CopilotError`, are now
  `#[non_exhaustive]`. Downstream matches must include a wildcard arm.
- **Breaking:** Event envelopes and payload structs, and inbound session, model,
  status, auth-status, agent, tool, quota, shell, and workspace response/metadata
  structs are now `#[non_exhaustive]`. Use helpers, available defaults, or Serde
  deserialization instead of struct literals, and `..` in struct patterns.
  Configuration and request struct literals remain supported. JSON wire formats
  and unknown-event handling are unchanged.

### Fixed

- Preserve raw JSON payloads when known session events fail typed decoding.

## 3.1.1 - 2026-08-15

### Added

- Initial changelog entry for the current repository version.
