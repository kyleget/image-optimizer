# 13: Prove native offline release artifacts

**What to build:** self-contained native release artifacts for every initial platform, validated through the public CLI under the conditions in which developers and agents will actually use them.

**Blocked by:** 12: Bundle the agent workflow skill.

- [ ] Release builds produce native binaries for macOS ARM64, macOS x64, and Linux x64 glibc with all required codec dependencies linked and no separately installed codec executable required.
- [ ] The release-relevant integration suite runs against each built artifact through the public CLI rather than substituting private implementation seams.
- [ ] Installed artifacts complete representative inspect-review-apply-clean workflows with network access unavailable.
- [ ] The cross-platform fixture matrix covers supported static and animated inputs, misleading extensions, transparency, orientation, color, metadata, malformed files, recursive traversal, symlinks, already-optimized files, resize modes, stale and corrupt candidates, destination conflicts, permissions, cleanup, and every resource limit.
- [ ] Tests verify decoded-pixel equality for lossless compression-only candidates, the stated invariants for pixel-changing candidates, exact application of inspected bytes, stream separation, schema and reason-code stability, summary counts, and exit statuses.
- [ ] Release validation does not require Linux ARM64, musl Linux, Windows, mobile, WebAssembly, release hosting, or installer-channel selection.
