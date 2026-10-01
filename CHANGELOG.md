# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows the versioning policy documented in
[`docs/semver-policy.md`](docs/semver-policy.md).

## Unreleased

### Changed

- **Breaking:** raised the minimum supported Rust version and pinned development
  and CI toolchains from Rust 1.85 to Rust 1.99.0. Edition 2021 is unchanged; see
  the [MSRV audit and edition decision](docs/msrv-policy.md).

### Added

- Added release policy documentation for changelog, semver, and MSRV handling.
- Added `CopilotError::EventsLagged(u64)` to report lost session events.

### Fixed

- Preserve raw JSON payloads when known session events fail typed decoding.
- Subscribe before sending in `send_and_collect` so early response and idle events
  are retained, and avoid duplicating streamed deltas with full message content.
- Return an error on event lag in `send_and_collect` and `wait_for_idle` instead
  of silently returning potentially incomplete results.

## 3.1.1 - 2026-08-15

### Added

- Initial changelog entry for the current repository version.
