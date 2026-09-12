# Ship a native Rust CLI with internal codec adapters

The image optimizer will be implemented in Rust and distributed as native
macOS and Linux binaries, with JPEG, PNG, and WebP behavior hidden behind
format-specific codec adapters. This keeps the agent-facing CLI independent of
runtime package managers and separately installed tools while allowing each
format to use the backend needed for decoded-pixel and metadata guarantees.

## Status

Accepted

## Considered Options

- A Node.js CLI with a general image library simplified pixel-changing
  operations but still required specialized behavior for decoded-pixel-preserving
  JPEG optimization.
- A thin wrapper around system executables reduced implementation effort but
  made installation, versions, and behavior host-dependent.
- A native Rust binary with internal adapters requires more integration work but
  gives callers one stable executable and localizes codec-specific complexity.

## Consequences

Release engineering must build and test native artifacts for macOS ARM64/x64
and Linux x64 glibc. Codec dependencies must link into those artifacts, and each
adapter must prove the product's format, pixel, metadata, and resource-safety
invariants before release. Codec choices can change internally without changing
the public CLI contract.
