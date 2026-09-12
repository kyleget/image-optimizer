# 08: Apply inspected candidates safely

**What to build:** intentional application of exact inspected candidate bytes to their protected source destinations, with stale or invalid selections isolated so valid work in the same batch can still complete.

**Blocked by:** 01: Inspect lossless PNG candidates.

- [ ] `apply` requires a session plus one or more candidate identifiers or an explicit all-candidates selection; ambiguous or empty selection is invalid invocation.
- [ ] The complete selection receives an item result, and candidate bytes and their manifest binding are validated before any selected item is written.
- [ ] Each source must match its inspected hash before staging and again immediately before replacement; a changed source is reported stale and remains untouched.
- [ ] Eligible writes stage a temporary file in the destination directory and use atomic replacement, and the resulting bytes exactly match the inspected candidate hash.
- [ ] A stale, missing, corrupt, or otherwise failed item does not prevent unrelated valid selections from being applied.
- [ ] Successful application removes candidate image bytes and retains a compact `applied` audit entry containing hashes, sizes, destination, and application time; unsuccessful and unapplied candidates remain available.
- [ ] Application emits one versioned JSON document with authoritative item outcomes `applied`, `stale`, or `failed`, stable reason codes, accurate summary counts, diagnostics on standard error, and the specified `0`, `1`, and `2` exit behavior.
- [ ] Focused race tests force a source change between the two hash checks and prove that changed bytes are never overwritten.
