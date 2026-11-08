# MSRV Policy

The minimum supported Rust version (MSRV) is Rust 1.99.0.

`Cargo.toml` records this as `rust-version = "1.99.0"`, and
`rust-toolchain.toml` pins the development toolchain to Rust 1.99.0.
Both CI jobs use the same pinned toolchain. Future increases must update all
three of the manifest, toolchain file, and this policy together.

## Edition decision

The crate and the detached code generator remain on edition 2021. Rust 1.99.0
supports edition 2024, but none of the toolchain upgrade requires an edition
migration. Keeping edition 2021 avoids combining an MSRV change with changes to
name resolution, temporary lifetimes, or public macro behavior. An edition 2024
migration should be reviewed separately; there is no Rust edition 2026.

## Rust 1.99.0 audit

Sources: [Rust release announcement](https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/),
[Rust release notes](https://doc.rust-lang.org/stable/releases.html#version-1990-2026-10-01),
[Cargo changelog](https://doc.rust-lang.org/nightly/cargo/CHANGELOG.html#cargo-199-2026-10-01),
and [Clippy changelog](https://github.com/rust-lang/rust-clippy/blob/master/CHANGELOG.md#rust-199).

- C-ABI variadic definitions (`extern "C"` and `"C-unwind"`) and the new
  raw-pointer layout APIs are not used by this crate. The library retains
  `#![forbid(unsafe_code)]`; these stabilizations do not make arbitrary FFI safe.
- The audit of library, examples, tests, and code generator found no `Box::leak`
  or other `leak` calls, `mem::forget`, or raw-pointer ownership round trips.
  No unleaking replacements are needed. Future ownership transfers should use
  `Box::into_non_null` or `Box::into_raw`, not reclaim a leaked reference.
- Cargo's new `debug` profile does not require changing our profiles.
  Incremental compilation is now disabled by default when `CI` is set.
  The edition-2024 change to inherited dependency `default-features` does not
  apply: the crate does not inherit workspace dependencies. There are no
  hyphenated lint names in manifest lint tables to migrate.
- Clippy 1.99 adds default-enabled `nonnull_unchecked_on_box_ptr`,
  `block_scrutinee`, and `mismatched_bit_width_type`; its new pedantic/restriction
  lints are not enabled here. Changes to `clone_on_copy`, `approx_constant`, and
  must-use analysis are covered by the all-targets/all-features Clippy check.
  The upgrade also exposes `derivable_impls` for `AutoModeSwitchResponse`;
  deriving `Default` with `No` as the default preserves the existing behavior.
  The snapshot test uses `std::io::Error::other` for `io_other_error`.
  The escape-room example uses `is_multiple_of` for `manual_is_multiple_of`,
  and the E2E calculator uses a match guard for `collapsible_match`.
  No new lint suppression is needed.

## Raising MSRV

An MSRV increase must:

- be intentional and called out in `CHANGELOG.md`;
- update `Cargo.toml`, `rust-toolchain.toml`, this policy, and any README or
  contributor guidance that mentions the Rust version;
- be treated as a compatibility-affecting change under
  [`docs/semver-policy.md`](semver-policy.md).
