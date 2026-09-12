# 12: Bundle the agent workflow skill

**What to build:** a bundled skill that lets an agent install and operate the optimizer safely through the intended inspect-review-apply workflow and explain the results accurately.

**Blocked by:** 09: Protect and apply variant destinations; 10: Clean retained sessions; 11: Enforce resource limits and controlled concurrency.

- [ ] The skill explains supported installation, offline and local-processing guarantees, JSON and standard-error handling, schema-version compatibility, reason codes, summaries, and exit statuses.
- [ ] It guides an agent through inspection, review of exact sizes and dimensions, selective or explicit whole-session application, verification of resulting files, and cleanup.
- [ ] Preset and metadata guidance states the actual tradeoffs and never describes resizing, glossy, or lossy work as lossless.
- [ ] Dimension guidance uses application-rendered size and display density to help the caller choose explicit bounds without claiming to infer them automatically.
- [ ] It explains intentional skips, per-file failures, stale candidates, destination protection, unchanged results, and how mixed batches must be interpreted.
- [ ] Representative workflow tests use real project images in JPEG, PNG, and WebP and verify that the documented commands and interpretations agree with the released public CLI.
