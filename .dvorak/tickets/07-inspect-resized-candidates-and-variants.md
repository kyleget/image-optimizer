# 07: Inspect resized candidates and variants

**What to build:** safe, explicit resize inspection that fits supported images within caller-supplied bounds and can target either an intentional variant path or an explicitly chosen source replacement.

**Blocked by:** 02: Inspect lossless JPEG candidates; 03: Inspect lossless WebP candidates.

- [ ] Callers may supply maximum width, maximum height, or both; output fits every supplied bound using documented deterministic integer rounding and preserves aspect ratio.
- [ ] Resizing never upscales, crops, stretches, or changes the input format, and before-and-after dimensions are reported accurately.
- [ ] Resizing is always reported as pixel-changing, including when combined with the lossless preset, and orientation and pixel-dependent metadata are normalized as specified.
- [ ] A resize requires either an explicit variant destination or an explicit choice to target the source; an existing variant can be targeted only through an explicit replacement choice.
- [ ] Inspection records canonical source and destination identities and the inspected destination state but changes neither file.
- [ ] A resized result is retained only when it is valid and strictly smaller than the inspected source; a valid non-smaller result is reported unchanged and cannot be applied.
- [ ] Fixtures cover width-only, height-only, combined bounds, rounding, no-op bounds, transparency, source replacement, new variants, existing variants, and destination collisions.
