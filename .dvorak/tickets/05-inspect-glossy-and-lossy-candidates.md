# 05: Inspect glossy and lossy candidates

**What to build:** explicit glossy and lossy inspection across JPEG, PNG, and WebP, using centrally owned versioned policies and truthfully reporting every pixel-changing operation.

**Blocked by:** 02: Inspect lossless JPEG candidates; 03: Inspect lossless WebP candidates.

- [ ] `inspect` accepts only `lossless`, `glossy`, and `lossy`, defaults to lossless, and rejects arbitrary encoder controls as invalid invocation.
- [ ] Glossy and lossy resolve centrally to versioned format-specific policies that are recorded along with the actual operations for each supported item.
- [ ] Candidates preserve their input format, transparency behavior, displayed orientation, color appearance, and applicable metadata policy, and decode successfully after encoding.
- [ ] Pixel-changing processing is never described as lossless, and only a valid candidate strictly smaller than its inspected source is retained as eligible for application.
- [ ] Tests cover each preset and format, including transparent images, orientation and color information, metadata-bearing files, and valid results that fail to save bytes.
- [ ] Acceptance does not depend on a perceptual score, a representative visual corpus, or equivalence to a third-party service; reports and documentation make no stronger fidelity claim than the Spec permits.
