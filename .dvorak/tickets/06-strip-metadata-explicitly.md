# 06: Strip metadata explicitly

**What to build:** an explicit metadata-removal choice that removes nonessential information while keeping optimized images correctly oriented and color-managed.

**Blocked by:** 02: Inspect lossless JPEG candidates; 03: Inspect lossless WebP candidates.

- [ ] Metadata preservation remains the default for JPEG, PNG, and WebP, and removal occurs only when the caller explicitly requests it.
- [ ] Removal strips nonessential supported metadata while retaining or normalizing information needed for correct displayed orientation and color appearance.
- [ ] The selected metadata policy and the metadata operations actually performed are represented in each applicable result and persisted candidate manifest.
- [ ] Metadata stripping is treated as an explicit operation and a retained result must still be valid and strictly smaller than its inspected source.
- [ ] Fixtures verify retained appearance and intended removal for supported metadata classes in all three formats without changing the source during inspection.
