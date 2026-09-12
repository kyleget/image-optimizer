# Image Optimizer Architecture

## Context

The product is an agent-first, offline CLI that inspects JPEG, PNG, and WebP
files, materializes reviewable candidates, and later applies exactly those
candidates without overwriting a changed source. Delivery needs guidance beyond
the Spec because decoded-pixel-preserving optimization, pixel-changing presets,
metadata handling, and safe filesystem replacement require different codec
behavior while the caller needs one small, stable interface.

The first release targets native macOS ARM64 and x64 binaries and a native
glibc-based Linux x64 binary. Linux ARM64 and musl builds are deferred.

## Decisions

- Implement the CLI in Rust and distribute platform binaries. Optimization must
  require no network access and no separately installed executables at runtime.
- Expose `inspect`, `apply`, and `clean` commands. Every command emits one
  versioned JSON document on stdout by default; diagnostics go to stderr. A
  `--human` rendering may be provided without changing the JSON contract.
- `inspect` stores immutable candidates and a versioned manifest in the
  operating-system cache, with an optional caller-supplied candidate directory.
  It returns opaque session and candidate identifiers. The manifest records
  canonical source and destination paths, cryptographic hashes of source and
  candidate bytes, dimensions, byte sizes, selected operations, resolved preset
  policy, and per-file outcome.
- `apply` requires candidate identifiers or an explicit `--all`. It validates
  the complete selection before writing, then applies valid entries
  independently. A stale or invalid entry remains untouched and does not prevent
  unrelated valid entries from being applied.
- Each file replacement uses a temporary file in the destination directory and
  an atomic rename. Validate the source hash before staging and again immediately
  before replacement. A variant destination that already exists is a failure
  unless inspection explicitly recorded replacement of that same destination.
- Delete candidate bytes immediately after successful application. Retain the
  small manifest entry as an audit ledger marked `applied`, including hashes,
  sizes, destination, and application time. Failed, stale, and unapplied
  candidates remain. `clean` removes a session or sessions older than a supplied
  duration.
- Do not follow directory symlinks and report all symlink inputs as skipped. This
  keeps source identity and recursive traversal within the requested tree.
- Exit `0` when every item is successful, unchanged, or intentionally skipped;
  exit `1` when any item fails, is stale, or cannot be applied safely; and exit
  `2` for invalid invocation. The JSON document remains the detailed authority
  for mixed batches.
- Expose only the `lossless`, `glossy`, and `lossy` presets. Map them centrally to
  versioned, format-specific policies and report the resolved operations. Do not
  expose arbitrary codec flags.
- For lossless optimization without resizing, preserve all supported metadata.
  For resizing or lossy processing, preserve descriptive metadata and color
  profiles, normalize orientation into pixels, and remove or regenerate
  pixel-dependent fields such as dimensions, orientation, and embedded
  thumbnails.
- Enforce configurable decoded-pixel, input-byte, memory/concurrency, and codec
  time limits. Released defaults are part of the preset/runtime policy. A limit
  violation is a per-file failure rather than a batch abort.
- Glossy and lossy validation requires only a smaller, decodable candidate plus
  the general format, transparency, orientation, color, and metadata invariants.
  Representative visual review and perceptual-quality thresholds are explicitly
  not release gates. This accepts the risk that the preset names express intent
  without verifying perceived fidelity.

## Interfaces and Seams

The public CLI is the external seam and the only interface agents need to learn.
It translates command input into requests to one deep optimization module and
renders returned reports; it contains no codec, persistence, or replacement
policy.

The optimization module owns discovery, format detection, animation rejection,
resize planning, preset resolution, metadata policy, candidate validation,
session persistence, resource limits, and safe application. Its small interface
is conceptually `inspect(request) -> report`, `apply(request) -> report`, and
`clean(request) -> report`. Concrete Rust types and exact flag spelling belong in
the Spec or implementation, not this design.

Format-specific codec adapters form a real internal seam because JPEG, PNG, and
WebP require materially different lossless and metadata operations. Adapters
accept resolved operations and return encoded candidate bytes plus observed
properties. They do not choose product presets, write source files, manage
sessions, or decide whether a candidate is eligible. The initial adapters must
be linked into the released binary; backend selection is not a caller-visible
plugin system.

Session storage and filesystem replacement remain private implementations of the
optimization module. They may have internal seams for deterministic tests, but
callers must not coordinate manifest files, hashes, temporary files, or codec
adapters themselves.

## Testing Seam

Invoke the released CLI with real fixture files and assert its versioned JSON,
exit status, candidate/session effects, and resulting files. Tests cross the same
external seam as agents.

The fixture matrix covers static and animated inputs, all supported formats,
transparent pixels, orientation and color profiles, relevant metadata classes,
already-optimized files, malformed files, resource-limit violations, recursive
inputs, symlinks, destination collisions, and changed sources. For default
lossless processing without resizing, compare decoded pixels and required
metadata. For resizing, assert dimensions and display-affecting properties. For
glossy and lossy, assert only valid decoding, smaller size, and the general
preservation invariants; visual quality is not an acceptance criterion.

Internal tests may replace codec and filesystem dependencies at their private
seams to force failures and races, but they do not substitute for cross-platform
CLI integration tests. Run the relevant suite against macOS ARM64/x64 and Linux
x64 glibc release artifacts, including offline execution after installation.

## Deferrals

- Exact codec crates, FFI libraries, encoder constants, resize filter, and preset
  policy version format are deferred behind the format-adapter seam. This is safe
  only if compatibility spikes prove decoded-pixel and metadata invariants before
  an adapter is accepted. Current evidence shows viable in-process directions:
  TurboJPEG exposes coefficient-domain lossless transforms, and Oxipng exposes a
  Rust library interface.
- Exact JSON field names, CLI option spelling, cache-path conventions, default
  resource limits, and concurrency counts are deferred to the Candidate Spec.
  Their governing semantics are settled above.
- Release hosting and installer channels are deferred until a Git host exists.
  The required release artifacts and runtime-independence contract are already
  fixed.
- Linux ARM64 and musl builds are deferred because they add native build and test
  combinations without serving the confirmed initial hosts.
