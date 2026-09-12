# 11: Enforce resource limits and controlled concurrency

**What to build:** bounded optimization that remains useful on mixed batches when malformed or extreme images exceed configured input, decoding, memory, concurrency, or time budgets.

**Blocked by:** 04: Inspect recursive and mixed input batches; 05: Inspect glossy and lossy candidates; 06: Strip metadata explicitly; 07: Inspect resized candidates and variants.

- [ ] Released defaults are 512 MiB input bytes per file, 100 million decoded pixels per file, 1 GiB aggregate working memory, no more than four concurrent codec jobs, and 120 seconds of codec time per file.
- [ ] Every limit is configurable per invocation, validated before work begins where possible, and represented in the resolved runtime policy needed to reproduce or audit an inspection.
- [ ] Effective concurrency is reduced when necessary to remain within the aggregate memory budget, and work is never admitted when its accounted memory would exceed that budget.
- [ ] Exceeding a per-file size, pixel, memory, or time limit produces a stable per-file failure without aborting unrelated batch items.
- [ ] Resource enforcement covers lossless, glossy, lossy, metadata-removal, and resize paths for every supported adapter.
- [ ] Deterministic tests exercise each limit, concurrency reduction, codec timeout, malformed expansion, mixed batches, accurate summaries, and exit status `1` for batches containing limit failures.
