# 04: Inspect recursive and mixed input batches

**What to build:** resilient batch inspection for multiple file and directory inputs so callers can review every discovered item and understand all intentional omissions and failures without losing valid work.

**Blocked by:** 01: Inspect lossless PNG candidates.

- [ ] `inspect` accepts one or more file or directory inputs, walks directories recursively, and processes ordinary files whether or not they are tracked by Git.
- [ ] Explicit symlink inputs and symlinks encountered during traversal are never followed and are reported as skipped with stable reason codes.
- [ ] Unsupported formats, animated inputs supported by an available adapter, duplicate discoveries, malformed inputs, and filesystem failures have explicit per-item outcomes and reason codes.
- [ ] One skipped or failed item does not prevent unrelated valid files from being inspected and retained as candidates where eligible.
- [ ] The summary counts agree with the item outcomes, mixed batches use exit status `1` if any item fails, and batches containing only successes, unchanged items, or intentional skips use exit status `0`.
- [ ] Candidate and session identities remain unambiguous when input paths overlap or the same file is discovered more than once.
