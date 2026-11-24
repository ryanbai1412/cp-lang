# ioi25_triples

Score: 4/5
Dream: 4/5

Re-reviewed cold on cdc542dcc against the statement (IOI 2025 Day 1, Triple Peaks), `dream.cp`
and `port.cp`, after the fbed8d10d review. Both owed rewrites are settled, and `verify` gives 0
findings in every mode (case 08 has no interpreter verdict past 20 s, as before).

Dream 4/5. The header derives both parts, including the √(6M) size and the jitter. The body
names the ideal forms: a `candidates` generator, `group.pairs()`, a `[0..m]?Int` table with
`h.map(v => v ?? ...)`, and `(0..8e5 / m).map(attempt).max(by: count_triples)!`. Counting is
`count(mythical(h, c) for c in candidates(h))`, with no shared counter. The only gap left is
reported: positions are raw `Int` inside `count_triples` (friction "The `1..n` height bound
drops at `count_triples`").

Port 4/5. It is correct and fast (case 08 0.56-0.75 s against orig.cpp's 1.61-2.49 s). The peak
table now takes the dream's form, `var h: [0..m]?Int = None` and `h.map(v => v ?? rng.int(1, 2))`
(`port.cp:80-83`), so the `m` sentinel is gone. `fn try` adding to a captured `found`
(`port.cp:29-65`) still stands in for the dream's generator, but the reason is now measured and
reported: the generator costs 1.7-1.9x at case 08, and under `-DLOCAL` its slowest run is past
the 2000 ms limit (friction "Counting from the `candidates` generator is 1.7-1.9x slower than
`fn try`", routed to codegen). That is a reported workaround, not an owed rewrite, so it holds
the port at 4 rather than 5.

## Checklist failures

None.

## Owed rewrites

None. Take the dream's `count(mythical(h, c) for c in candidates(h))` once codegen inlines a
`yields` fn into its only fold consumer (the friction's general fix).

## Language gaps

Reported: `pairs()` materialising (L0543 is kept on purpose), the `1..n` bound at
`count_triples`, the per-candidate list sort (the reason for the arithmetic test), the
`-DLOCAL` overflow checks under `cpl test`, and the generator's callback cost against `fn try`.
Two earlier gaps are marked as no longer reproducing (the closure-local copy and the
`max(by:)!.1` copy).
