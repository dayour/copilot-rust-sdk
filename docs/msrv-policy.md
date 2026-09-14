# MSRV Policy

The minimum supported Rust version (MSRV) is Rust 1.85.

`Cargo.toml` records this as `rust-version = "1.85"`, and
`rust-toolchain.toml` currently pins the development toolchain to Rust 1.85.0.
The MSRV remains Rust 1.85 until maintainers explicitly raise it in both the
package metadata and this policy.

## Raising MSRV

An MSRV increase must:

- be intentional and called out in `CHANGELOG.md`;
- update `Cargo.toml`, `rust-toolchain.toml`, this policy, and any README or
  contributor guidance that mentions the Rust version;
- be treated as a compatibility-affecting change under
  [`docs/semver-policy.md`](semver-policy.md).
