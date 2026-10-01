# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows the versioning policy documented in
[`docs/semver-policy.md`](docs/semver-policy.md).

## Unreleased

### Added

- Added release policy documentation for changelog, semver, and MSRV handling.

### Fixed

- Preserve raw JSON payloads when known session events fail typed decoding.
- Normalize RPC transport disconnects to `ConnectionClosed` and explicit stops to
  `Shutdown`, including pending requests and calls after disconnect. Genuine server
  JSON-RPC errors are preserved instead of being treated as synthetic disconnects.
- Monitor owned CLI processes and report `ProcessExit` with the observed exit code,
  including exits while the transport remains open.
- Count and log malformed inbound RPC messages without logging their payloads.

## 3.1.1 - 2026-08-15

### Added

- Initial changelog entry for the current repository version.
