## Problem Statement

Images in websites and apps can consume unnecessary bytes or exceed the resolution needed by the application. During development, an AI agent needs a straightforward way to inspect and reduce those assets without a graphical interface, unexpected quality changes, or unsafe replacement of source files.

The initial user is the project owner working on macOS and a headless Linux host. Optimized images remain ordinary project files that are committed and deployed normally.

## Solution

Provide a local CLI and an accompanying agent skill. The CLI operates offline after installation, accepts files or recursively scans directories, and supports JPEG, PNG, and WebP while preserving the input format.

Inspection generates optimized candidates and reports exact dimensions and byte sizes without replacing source files. An explicit apply operation uses those candidates, protects originals changed since inspection, and writes only smaller results.

Lossless compression is the default. Glossy and lossy are explicit, documented presets prioritizing visual fidelity and file size respectively; they need not reproduce ShortPixel's results. Optional resizing fits within supplied maximum dimensions without upscaling, cropping, or stretching. Resizing changes pixels even when followed by lossless compression.

## User Stories

1. As a developer, I want to optimize project images during development, so that deployed assets use fewer bytes.
2. As a developer, I want a local CLI, so that I can use it without a graphical interface.
3. As a developer, I want macOS support, so that I can optimize images on my workstation.
4. As a developer, I want Linux support, so that I can run the tool on a headless host.
5. As a developer, I want offline processing after installation, so that optimization requires no external service.
6. As a developer, I want images processed locally, so that source images need not be uploaded.
7. As an agent, I want a bundled usage skill, so that I can follow the intended workflow.
8. As an agent, I want structured results, so that I can interpret outcomes reliably.
9. As an agent, I want to inspect individual files, so that I can target specific assets.
10. As an agent, I want recursive directory inspection, so that I can optimize a collection of assets.
11. As a developer, I want to process uncommitted images, so that new assets can be optimized before their first commit.
12. As a developer, I want JPEG support, so that I can optimize photographic assets.
13. As a developer, I want PNG support, so that I can optimize graphics with transparency.
14. As a developer, I want WebP support, so that I can optimize existing WebP assets.
15. As a developer, I want input formats preserved, so that optimization does not require changing asset references.
16. As a developer, I want lossless compression by default, so that compression preserves decoded pixels.
17. As a developer, I want an explicit glossy preset, so that I can prioritize visual fidelity while accepting pixel changes.
18. As a developer, I want an explicit lossy preset, so that I can prioritize smaller files while accepting quality changes.
19. As an agent, I want clear preset descriptions, so that I do not claim all modes are lossless.
20. As a developer, I want metadata preserved by default, so that optimization does not silently remove information.
21. As a developer, I want explicit metadata removal, so that I can choose to strip metadata.
22. As a developer, I want orientation and color appearance preserved, so that optimized assets display correctly.
23. As a developer, I want transparency preserved, so that transparent assets retain their intended appearance.
24. As an agent, I want exact candidate byte sizes, so that I can explain savings before applying changes.
25. As an agent, I want before and after dimensions, so that I can explain resizing effects.
26. As a developer, I want inspection to leave originals intact, so that I can review proposed changes.
27. As a developer, I want explicit application of inspected candidates, so that the applied result is the reviewed result.
28. As a developer, I want changed originals protected, so that applying an older candidate cannot erase subsequent edits.
29. As a developer, I want non-smaller results left unapplied, so that optimization never increases file size.
30. As an agent, I want unsupported and animated files reported as skipped, so that omissions are visible.
31. As an agent, I want individual failures reported, so that I can distinguish them from successful and unchanged files.
32. As a developer, I want optional maximum width and height, so that I can fit images to application needs.
33. As a developer, I want aspect ratio preserved without upscaling, so that resizing avoids distortion and unnecessary enlargement.
34. As an agent, I want guidance on rendered size and display pixel density, so that I can choose appropriate explicit dimensions.
35. As a developer, I want resized variants written to an explicit output path, so that I can retain source assets.
36. As a developer, I want replacement during resizing to be explicit, so that resizing does not unexpectedly replace originals.
37. As a developer, I want optimized files committed and deployed normally, so that application runtime has no dependency on the optimizer.

## Implementation Decisions

- Build one optimization module with a CLI interface for inspection and application. Keep encoding, preset selection, metadata handling, resizing, and candidate validation behind this interface.
- Provide an agent skill describing the CLI workflow, structured results, quality tradeoffs, and application-aware dimension selection.
- Support macOS and headless Linux from the first release. No network access is required during optimization after dependencies are installed.
- Accept individual files and recursive directory inputs regardless of Git tracking status.
- Support static JPEG, PNG, and WebP, preserving input format. Detect and skip animated images without flattening them.
- Default to lossless compression. Without resizing or explicit metadata removal, preserve decoded pixels, transparency, and metadata. Internal encoding may change. Report unchanged when no valid smaller candidate is available.
- Glossy and lossy require explicit selection and are the product's own presets. Their exact encoder settings remain an implementation decision requiring documented behavior and validation, not a promise of ShortPixel equivalence.
- Preserve metadata by default. Explicit stripping must still preserve correct orientation and color appearance; data needed to maintain those properties must not be blindly discarded.
- Inspection materializes candidates and reports original and candidate sizes, savings, dimensions, selected operations, and per-file outcomes in structured form.
- Application uses the inspected candidate rather than independently recompressing the image. Bind candidates to their inspected sources and validate that sources have not changed before replacement.
- Apply only candidates smaller than their source, including resized variants. Report valid results without savings as unchanged.
- Resizing is explicit and accepts maximum width, maximum height, or both. Fit inside supplied limits while preserving aspect ratio; do not upscale, crop, or stretch.
- The agent supplies dimensions using application context. The CLI does not inspect application code.
- Resized variants support an explicit output destination; replacing the original requires an explicit choice and uses the same inspection and application protections.
- Resizing is a pixel-changing operation regardless of the compression preset. Reports and skill guidance must make this distinction clear.
- Keep optimized images as ordinary files; do not add runtime serving or deployment integration.
- The repository currently has no implementation, domain glossary content, or established testing framework. Language, encoder dependencies, packaging, and exact command syntax remain implementation choices subject to these requirements.

## Testing Decisions

- Confirmed primary test seam: invoke the public CLI with real fixture files and inspect structured results and resulting files.
- Test externally observable behavior rather than private encoder calls or module structure. Exercise the optimization module through the same interface used by agents.
- Run relevant integration checks on both macOS and headless Linux, including offline execution after installation.
- For lossless compression without resizing, compare decoded pixels and verify transparency, metadata, orientation, and color handling across supported formats. Include already optimized files that cannot shrink.
- Verify explicit quality preset selection, input-format preservation, and valid decodable output. Use representative visual review for glossy and lossy; smaller byte size alone does not establish acceptable visual quality.
- Verify fitting by width, height, and both; aspect-ratio preservation with necessary integer rounding; no upscaling; and accurate reported dimensions.
- Verify inspection leaves source files unchanged, application writes the inspected candidate, changed originals are refused, and non-smaller candidates are not applied.
- Verify resized output destinations and explicit source replacement follow the agreed behavior.
- Verify recursive batches report successes, unchanged files, unsupported files, animated files, and failures individually.
- Validate the documented skill workflow against representative project images: inspect, explain savings and skips, then explicitly apply selected candidates.
- There is no existing test prior art in this repository. Select the test runner with the implementation stack.

## Out of Scope

- Graphical UI, public hosted API, accounts, billing, and multi-user service operation.
- Runtime image serving, CDN functionality, and automatic deployment or Git commits of optimized images.
- External optimization APIs or uploading source images.
- Format conversion, formats beyond JPEG/PNG/WebP, and animated-image optimization.
- Upscaling, cropping, stretching, and automatic inspection of application code by the CLI.
- Guaranteed savings for every image, unchanged pixels after resizing, or exact equivalence to ShortPixel presets.

## Further Notes

The user accepted the scope through discovery and grilling, including explicit resizing and Linux support, and confirmed the public CLI as the primary test seam.

No numerical savings target or representative image corpus has been supplied. Adoption is based on completing the agent workflow safely on both platforms with demonstrated savings where available, rather than a universal compression percentage.
