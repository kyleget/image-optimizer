## Problem Statement

Images in websites and applications often contain unnecessary bytes or exceed
the resolution the application displays. During development, a project owner or
AI agent needs a straightforward way to inspect and reduce those assets without
a graphical interface, network service, unexpected visual changes, or unsafe
replacement of source files.

The initial user works on macOS and on a headless Linux host. Optimized images
must remain ordinary project files that can be reviewed, committed, and deployed
through the project's existing workflow.

## Solution

Provide a native `image-optimizer` CLI and an accompanying agent skill. After
installation, optimization runs entirely locally. The CLI accepts individual
files or recursively scans directories, supports static JPEG, PNG, and WebP, and
preserves each input's format.

The workflow separates inspection from application. Inspection leaves source
and destination files untouched, materializes an immutable smaller candidate,
and reports exact byte sizes, dimensions, selected operations, and per-file
outcomes. Application writes that exact candidate only after revalidating its
source and destination, so a file changed after inspection is never silently
overwritten.

Lossless compression is the default. Explicit `glossy` and `lossy` presets
permit pixel changes in exchange for progressively stronger size reduction.
Optional resizing fits within explicit maximum dimensions without upscaling,
cropping, or stretching. Resizing is always reported as a pixel-changing
operation, even when combined with the lossless preset.

## User Stories

1. As a developer, I want to optimize project images during development, so that deployed assets use fewer bytes.
2. As a developer, I want a local CLI, so that I can optimize images without a graphical interface.
3. As a developer, I want native macOS support, so that I can optimize images on my workstation.
4. As a developer, I want native Linux support, so that I can optimize images on a headless host.
5. As a developer, I want offline processing after installation, so that optimization does not depend on an external service.
6. As a developer, I want images processed locally, so that source assets are never uploaded.
7. As an agent, I want a bundled usage skill, so that I can follow the intended inspect-review-apply workflow.
8. As an agent, I want versioned structured results, so that I can interpret outcomes reliably.
9. As an agent, I want diagnostics separated from structured output, so that human-readable messages do not corrupt machine-readable results.
10. As an agent, I want stable exit-status semantics, so that I can distinguish a clean batch, per-file problems, and invalid invocation.
11. As an agent, I want to inspect individual files, so that I can target specific assets.
12. As an agent, I want recursive directory inspection, so that I can optimize a collection of assets.
13. As a developer, I want symlinks skipped visibly, so that recursive optimization does not escape or ambiguously traverse the requested tree.
14. As a developer, I want to process uncommitted images, so that new assets can be optimized before their first commit.
15. As a developer, I want JPEG support, so that I can optimize photographic assets.
16. As a developer, I want PNG support, so that I can optimize graphics with transparency.
17. As a developer, I want WebP support, so that I can optimize existing WebP assets.
18. As a developer, I want input formats preserved, so that optimization does not require changing asset references.
19. As a developer, I want animated images skipped rather than flattened, so that motion is never destroyed silently.
20. As a developer, I want lossless compression by default, so that compression alone preserves decoded pixels.
21. As a developer, I want an explicit glossy preset, so that I can prioritize visual fidelity while accepting pixel changes.
22. As a developer, I want an explicit lossy preset, so that I can prioritize smaller files while accepting stronger quality changes.
23. As an agent, I want resolved preset behavior reported, so that I do not claim all operations are lossless.
24. As a developer, I want metadata preserved by default, so that optimization does not silently remove information.
25. As a developer, I want explicit metadata removal, so that I can choose to strip nonessential information.
26. As a developer, I want orientation and color appearance preserved, so that optimized assets display correctly.
27. As a developer, I want transparency preserved, so that transparent assets retain their intended appearance.
28. As an agent, I want exact candidate byte sizes and savings, so that I can explain the benefit before applying changes.
29. As an agent, I want before-and-after dimensions, so that I can explain resizing effects.
30. As a developer, I want inspection to leave source and destination files intact, so that I can review proposed changes safely.
31. As a developer, I want explicit application of inspected candidates, so that the applied bytes are the reviewed bytes.
32. As a developer, I want to select candidates individually or explicitly select an entire session, so that batch application is intentional.
33. As a developer, I want changed sources and destinations protected, so that an older candidate cannot erase subsequent work.
34. As a developer, I want each valid batch item applied independently, so that one stale or failed item does not discard unrelated valid work.
35. As a developer, I want non-smaller results left unapplied, so that optimization never increases file size.
36. As an agent, I want unsupported, animated, and symlink inputs reported as skipped, so that omissions are visible.
37. As an agent, I want malformed inputs, limit violations, stale candidates, and filesystem problems reported individually, so that I can distinguish them from successful and unchanged files.
38. As a developer, I want optional maximum width and height, so that I can fit images to application needs.
39. As a developer, I want aspect ratio preserved without upscaling, so that resizing avoids distortion and unnecessary enlargement.
40. As an agent, I want guidance on rendered size and display density, so that I can choose appropriate explicit dimensions.
41. As a developer, I want resized variants written to an explicit destination, so that I can retain source assets.
42. As a developer, I want replacement during resizing to be explicit, so that a pixel-changing operation does not unexpectedly replace the source.
43. As a developer, I want an existing variant destination protected unless replacement was explicitly inspected, so that applying a candidate cannot clobber unrelated output.
44. As a developer, I want resource limits, so that malformed or extreme images cannot consume unbounded time or memory.
45. As a developer, I want successful candidate bytes cleaned up while retaining a small audit record, so that the cache remains useful without retaining redundant image data.
46. As a developer, I want to clean abandoned sessions explicitly, so that unapplied candidates do not accumulate indefinitely.
47. As a developer, I want optimized files committed and deployed normally, so that application runtime has no dependency on the optimizer.

## Implementation Decisions

- Implement a native Rust CLI distributed as self-contained platform binaries
  for macOS ARM64, macOS x64, and Linux x64 glibc. Optimization must require no
  network access or separately installed executable after installation.
- Follow the accepted native-CLI ADR and the Image Optimizer Architecture. Keep
  the public CLI independent of codec selection through internal format-specific
  adapters. Codec dependencies must be linked into release artifacts and prove
  the format, decoded-pixel, metadata, and resource-safety invariants before an
  adapter is accepted.
- Expose three commands: `inspect`, `apply`, and `clean`. Commands emit exactly
  one JSON document on standard output by default, write diagnostics to standard
  error, and may offer an explicit human-readable renderer without changing the
  JSON contract.
- `inspect` accepts one or more file or directory inputs. Directory inputs are
  recursive. Inputs do not need to be Git-tracked. Format detection is based on
  file contents; static JPEG, PNG, and WebP are supported. Unsupported formats,
  animated images, and all symlink inputs are skipped and reported.
- `inspect` accepts only the `lossless`, `glossy`, and `lossy` presets, defaulting
  to `lossless`. Presets resolve centrally to versioned, format-specific policies
  and arbitrary encoder flags are not exposed. The resolved policy and actual
  operations are reported for every supported input.
- `lossless` without resizing preserves decoded pixels, transparency, and all
  metadata the selected adapter supports. Internal encoding may change. An
  adapter that cannot preserve a present supported metadata class must fail the
  file rather than silently discard it.
- `glossy` and `lossy` explicitly permit pixel changes. They preserve the input
  format, transparency behavior, displayed orientation, color appearance, and
  applicable metadata policy. The presets are product-owned and make no promise
  of matching ShortPixel or another service.
- Metadata is preserved by default. Explicit stripping removes nonessential
  metadata while retaining or normalizing information required for correct
  orientation and color appearance. Resizing or lossy processing normalizes
  orientation into pixels and removes or regenerates pixel-dependent fields such
  as stored dimensions, orientation tags, and embedded thumbnails.
- Resizing accepts a maximum width, a maximum height, or both. It fits within all
  supplied limits, preserves aspect ratio using documented deterministic integer
  rounding, and never upscales, crops, or stretches. The report distinguishes
  compression-only operations from pixel-changing resize operations.
- Compression-only inspection targets the source path for later replacement.
  Resizing requires either an explicit variant destination or an explicit choice
  to target the source. A variant destination that exists at inspection time can
  be targeted only through an explicit replacement choice; its inspected bytes
  are then protected in the same way as source bytes.
- Successful inspection creates an immutable candidate and a versioned manifest
  in the operating-system cache. macOS uses its conventional per-user caches
  location. Linux uses `XDG_CACHE_HOME` when set and the conventional per-user
  cache location otherwise. A caller may supply a different candidate directory.
- Inspection reports a valid result without savings as `unchanged` and does not
  retain candidate bytes for it. Only a valid candidate strictly smaller than
  its inspected source is eligible for later application, including when
  resizing or writing a variant destination.
- Each inspection returns an opaque session identifier, and each retained
  candidate has an opaque identifier unique within that session. A manifest
  binds the candidate to canonical source and destination paths; source,
  destination, and candidate hashes; original and candidate sizes and
  dimensions; format; preset policy; operations; metadata policy; and outcome.
- `apply` requires a session identifier plus one or more candidate identifiers,
  or an explicit all-candidates selection. It reads the stored candidate rather
  than recompressing or resizing the source.
- Before writing, `apply` evaluates the complete selection and records a result
  for every selected item. It validates candidate integrity and revalidates the
  source and any pre-existing destination against the inspected hashes. A stale
  or invalid item remains untouched and does not prevent unrelated valid items
  from being applied.
- Each eligible write uses a temporary file in the destination directory. The
  source is hash-checked before staging and again immediately before an atomic
  rename. A destination that was absent during inspection must still be absent;
  a destination that was explicitly selected for replacement must still match
  its inspected hash.
- After successful application, delete the candidate image bytes and retain a
  small manifest ledger entry marked `applied`, including hashes, sizes,
  destination, and application time. Failed, stale, and unapplied candidates are
  retained until cleaned.
- `clean` removes explicitly selected sessions or sessions older than an
  explicitly supplied duration. It never removes source or applied destination
  files and reports each affected session.
- JSON output uses a top-level integer `schema_version`, the command name, an
  optional session identifier, a summary of outcome counts, and an `items`
  array. Consumers must ignore unknown fields. A breaking change to field meaning
  or required structure increments `schema_version`.
- Every item reports its outcome and a stable machine-readable reason code.
  Inspection outcomes are `candidate`, `unchanged`, `skipped`, or `failed`;
  application outcomes are `applied`, `stale`, or `failed`; cleanup outcomes are
  `cleaned`, `skipped`, or `failed`. Applicable items also report source and
  destination paths, candidate identifier, format, preset and operations,
  original and candidate byte sizes and dimensions, byte and percentage savings,
  and hashes. Inapplicable fields are omitted rather than populated with invented
  values.
- Exit status is `0` when every item succeeds, is unchanged, or is intentionally
  skipped; `1` when any item fails, is stale, or cannot be applied safely; and `2`
  for invalid invocation. JSON item outcomes remain authoritative for mixed
  batches.
- Enforce configurable limits per invocation. Initial defaults are 512 MiB input
  bytes per file, 100 million decoded pixels per file, 1 GiB aggregate working
  memory, at most four concurrent codec jobs, and 120 seconds of codec time per
  file. Effective concurrency may be lowered to respect the memory limit. A
  limit violation is a per-file failure and does not abort unrelated items.
- The bundled agent skill explains installation, structured-output handling,
  preset and metadata tradeoffs, application-aware dimension selection,
  inspect-review-apply usage, selective application, skip/failure handling, and
  cleanup. It must never describe resizing, glossy, or lossy processing as
  lossless.
- Optimized images remain ordinary files. The CLI does not alter Git state,
  application source, deployment configuration, or runtime asset delivery.
- Accepted material risk: glossy and lossy candidates must be smaller,
  decodable, and satisfy the format, transparency, orientation, color, and
  metadata invariants, but perceptual-quality thresholds and representative
  visual review are not release gates. The preset names therefore express
  product intent without objectively proving perceived fidelity.

## Testing Decisions

- The primary and highest practical test seam is the released public CLI. Invoke
  it with real fixture files and assert versioned JSON, standard streams, exit
  status, candidate/session effects, and resulting file bytes and properties.
- Tests assert externally observable behavior rather than private encoder calls
  or internal module structure. Focused internal tests may substitute codec and
  filesystem dependencies to force races and failures, but they do not replace
  CLI integration coverage.
- Run the release-relevant integration suite against macOS ARM64, macOS x64, and
  Linux x64 glibc artifacts. Verify optimization after installation with network
  access unavailable and without separately installed codec executables.
- Cover static and animated inputs, all supported formats, misleading file
  extensions, transparent pixels, orientation tags, color profiles, supported
  metadata classes, already-optimized files, malformed files, recursive inputs,
  symlinks, duplicate and colliding destinations, permission failures, and each
  resource limit.
- For lossless compression without resizing or metadata removal, compare decoded
  pixels exactly and verify preservation of transparency, supported metadata,
  displayed orientation, and color appearance. Include inputs for which no
  smaller valid encoding exists.
- For glossy and lossy presets, verify explicit selection, resolved-policy
  reporting, valid decoding, strict byte savings, input-format preservation,
  transparency behavior, displayed orientation, color appearance, and the
  selected metadata policy. Perceptual similarity is not an acceptance gate.
- For resizing, test width-only, height-only, and combined limits; deterministic
  integer rounding; no upscaling; aspect-ratio preservation; accurate dimensions;
  explicit variant output; explicit source replacement; and cases where a resized
  result is not smaller.
- Verify that inspection never changes a source or destination; application
  writes the inspected candidate bytes; changed sources, changed destinations,
  missing candidates, corrupt manifests, and corrupt candidate bytes are refused;
  and non-smaller candidates are never applied.
- Exercise source changes during application to verify the second hash check and
  atomic replacement behavior. Verify that a stale or failed item does not stop
  unrelated valid candidates in the same selection.
- Verify successful candidate-byte deletion, retained applied audit records,
  explicit session cleanup, age-based cleanup, and preservation of all project
  source and destination files during cleanup.
- Verify the JSON schema and reason codes for every outcome, one-document stdout
  behavior, stderr separation, summary counts, forward-compatible unknown fields,
  and exit statuses `0`, `1`, and `2`.
- Validate the documented agent-skill workflow against representative project
  images: inspect, explain candidates and skips, select candidates, apply them,
  and verify final files.
- There is no existing implementation or test prior art in this repository. Use
  the Rust test tooling selected with the implementation while preserving the
  public CLI as the release acceptance seam.

## Out of Scope

- A graphical interface, hosted API, accounts, billing, or multi-user service.
- Runtime image serving, CDN functionality, deployment automation, or automatic
  Git staging and commits.
- External optimization APIs, uploading source images, or network access during
  optimization.
- Format conversion, formats other than JPEG, PNG, and WebP, or optimization of
  animated images.
- Linux ARM64, musl-based Linux, Windows, mobile platforms, and WebAssembly in the
  first release.
- Upscaling, cropping, stretching, art-direction transforms, or automatic
  inspection of application code to choose dimensions.
- Public codec plugins, caller-selected encoder flags, or a promise to keep one
  codec backend permanently.
- Guaranteed savings for every image, unchanged pixels after resizing, exact
  equivalence to third-party presets, or a perceptual-quality release threshold.
- Release hosting and installer-channel selection until a Git host is available.

## Further Notes

The Image Optimizer Architecture governs module boundaries, session ownership,
safe filesystem replacement, codec adapters, and the public testing seam. The
accepted ADR “Ship a native Rust CLI with internal codec adapters” governs the
implementation language, native distribution model, and adapter consequences.

The product owner accepted explicit resizing, Linux support, the public CLI as
the primary test seam, and the material quality-validation risk during shaping.
No universal numerical savings target or representative image corpus has been
supplied. Adoption is based on completing the agent workflow safely on all
initial platforms and demonstrating savings where valid smaller candidates are
available.

Exact codec libraries, encoder constants, resize filter, and preset-policy
version format remain implementation choices behind the codec-adapter seam.
Compatibility spikes must prove the required decoded-pixel and metadata
invariants before an adapter is accepted.
