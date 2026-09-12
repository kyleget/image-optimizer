# 09: Protect and apply variant destinations

**What to build:** application of inspected resized variants with destination-state protection equivalent to source protection, including intentional replacement of a previously inspected destination.

**Blocked by:** 07: Inspect resized candidates and variants; 08: Apply inspected candidates safely.

- [ ] A destination absent during inspection must still be absent immediately before application, or the item is reported stale and no bytes at that path are replaced.
- [ ] A destination explicitly selected for replacement must still match its inspected hash before staging and immediately before atomic replacement.
- [ ] Applying a valid variant writes exactly the inspected candidate bytes while leaving its source asset unchanged.
- [ ] Source-targeted resizing replaces the source only when both source-replacement intent and all source revalidation checks are satisfied.
- [ ] Colliding destinations, changed destinations, permission failures, and mixed valid and invalid selections receive independent outcomes without discarding unrelated valid writes.
- [ ] Successful variant application performs the same candidate cleanup and retained audit-ledger update as source replacement.
