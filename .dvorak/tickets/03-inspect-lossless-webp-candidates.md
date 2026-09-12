# 03: Inspect lossless WebP candidates

**What to build:** lossless inspection for static WebP through the established candidate workflow, while detecting and visibly skipping animated WebP rather than flattening it.

**Blocked by:** 01: Inspect lossless PNG candidates.

- [ ] Static WebP is detected from its contents and uses the same public report, session, candidate, and manifest contract as the other supported formats.
- [ ] Lossless WebP inspection preserves decoded pixels exactly, transparency, displayed orientation, color appearance, and every present metadata class supported by the adapter.
- [ ] The adapter fails an item rather than silently losing a present supported metadata class and safely rejects malformed or extreme data.
- [ ] Animated WebP is reported as `skipped` with a stable reason code and is never flattened or retained as a candidate.
- [ ] Strictly smaller and valid non-smaller results follow the candidate and unchanged semantics respectively, and inspection leaves the source untouched.
