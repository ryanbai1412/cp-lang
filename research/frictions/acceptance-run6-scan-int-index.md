# Friction Report: acceptance run 2026-11-23-6, Int index into a scan

## Six accepted solutions index a `List` scan with a bare `Int`
- **Where:** research/acceptance/solutions/2026-11-23-6: linux/{cf2285D,cf2289D},
  macos/{cf2272E,cf2285D,cf2289D}, windows/cf2289D: `let sc = a.scan(+)`, then `sc[c]`,
  `sc[n]`, `sc[n - s]`, `pre[i]`.
- **What felt wrong:** nothing to the agent: unprompted, all six wrote `pre[k]` and meant
  "the fold of the first `k`" (cut `k`). Decision 507 (R07.1.6, E0320) now rejects that
  spelling, because on `Int` keys a position and a cut are both `Int`.
- **Goal it hurts:** familiar unless novelty removes a P0 bug class; writable from the docs.
  It is evidence on decision 507 for Ryan and Core to weigh: the off-by-one 507 prevents
  never happened in these six, while the error costs every one of them a fix.
- **Ideal code:** `pre.cut(k)`, which is what `cpl fix` writes, with the behaviour unchanged
  (std-prefix/scan-int-index-is-cut).
- **General fix:** keep E0320's machine fix so the migration is one `cpl fix`; decide on 507
  with this count in hand.
- **Severity:** medium
