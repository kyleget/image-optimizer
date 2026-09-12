# 02: Inspect lossless JPEG candidates

**What to build:** lossless JPEG inspection through the established candidate workflow, preserving the visible and metadata properties of the source while producing a reviewable candidate only when bytes are saved.

**Blocked by:** 01: Inspect lossless PNG candidates.

- [ ] JPEG is detected from file contents and processed through the same public report, session, candidate, and manifest contract as PNG.
- [ ] Lossless JPEG inspection preserves decoded pixels exactly, including displayed orientation and color appearance, and preserves every present metadata class supported by the selected adapter.
- [ ] The adapter fails an item rather than silently losing a present supported metadata class and safely rejects malformed or extreme data.
- [ ] A strictly smaller encoding is reported and retained as a candidate; a valid non-smaller encoding is reported as unchanged with no retained candidate bytes.
- [ ] Integration fixtures cover misleading extensions, orientation tags, color profiles, supported metadata, already-optimized images, and malformed JPEG data without modifying the inspected file.
