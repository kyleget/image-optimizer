# 10: Clean retained sessions

**What to build:** explicit cleanup of abandoned or aged optimizer sessions without ever removing project images or applied outputs.

**Blocked by:** 08: Apply inspected candidates safely.

- [ ] `clean` accepts explicitly selected sessions or an explicitly supplied age threshold and rejects an invocation with no cleanup scope.
- [ ] Cleanup removes retained candidate data and session state only for sessions selected by the caller or older than the supplied duration.
- [ ] Source files, variant destinations, source replacements, and every other project file remain untouched regardless of manifest corruption or partial session state.
- [ ] Applied audit information required by the retention contract remains small and intact when its surrounding session is cleaned.
- [ ] Every affected or requested session is reported as `cleaned`, `skipped`, or `failed` with a stable reason code, accurate summary counts, and the shared standard-stream and exit-status semantics.
- [ ] Tests cover explicit cleanup, age boundaries, active unapplied candidates, applied sessions, corrupt session data, and filesystem failures.
