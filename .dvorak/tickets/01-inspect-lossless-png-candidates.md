# 01: Inspect lossless PNG candidates

**What to build:** a native CLI that can inspect a single PNG through the complete lossless review path, reporting the result through the stable machine-readable contract and retaining an immutable candidate only when it is strictly smaller.

**Blocked by:** None (can start immediately).

- [ ] `inspect` identifies PNG content independently of the file extension and emits exactly one JSON document on standard output with an integer schema version, command, opaque session identifier, outcome summary, and item results; diagnostics do not contaminate standard output.
- [ ] A valid smaller PNG is reported as `candidate` with a stable reason code, opaque candidate identifier, canonical source and destination identities, hashes, exact byte sizes, dimensions, savings, resolved lossless policy, metadata policy, and actual operations.
- [ ] Candidate bytes are immutable and are stored with a versioned manifest in the conventional per-user cache location, with a caller-selected candidate directory supported.
- [ ] A valid result that is not strictly smaller is reported as `unchanged`, retains no candidate image bytes, and cannot later be applied.
- [ ] Lossless compression without resizing preserves decoded pixels exactly, including transparency, displayed orientation, color appearance, and every present metadata class supported by the adapter; the item fails instead of silently discarding supported metadata it cannot preserve.
- [ ] Inspection does not modify the source or destination, and malformed or unreadable PNG input is reported as a per-file failure rather than crashing or consuming resources without bound.
- [ ] Exit status is `0` for successful or unchanged inspection, `1` for a file failure, and `2` for invalid invocation, with stable machine-readable reason codes in item results.
