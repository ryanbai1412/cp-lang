# ioi25_triples

Score: 3/5
Dream: 4/5

Re-reviewed cold on fbed8d10d against the statement (IOI 2025 Day 1, Triple Peaks), `dream.cp`
and `port.cp`; the friction report was read only after scoring. The first review's dream
failures are fixed, so the dream is no longer sent back and the port score is final.

Dream 4/5. The header derives both parts, including the √(6M) size and the jitter. The body
names the ideal forms: a `candidates` generator, `group.pairs()`, a `[0..m]?Int` table with
`h.map(v => v ?? ...)`, and `(0..8e5 / m).map(attempt).max(by: count_triples)!`. Counting is
`count(mythical(h, c) for c in candidates(h))`, with no shared counter. The only gap left is
reported: positions are raw `Int` inside `count_triples` (friction "The `1..n` height bound
drops at `count_triples`").

Port 3/5. It is correct and fast (case 08 0.56-0.75 s against orig.cpp's 1.61-2.49 s), but it
keeps a workaround whose reason is gone.

## Checklist failures

None. The `1..n` bound the first review flagged is now in the friction report, and the needless
`.map(x => Int(x))` copy is gone from both programs.

## Owed rewrites

- `port.cp:80-84`: `var h: [0..m]Int = m` uses `m` as "no peak yet" (R5), filled in a loop and
  returned as `List(h)`. The friction report says the copy that forced it ("Mapping a closure's
  local table copies it") no longer reproduces on 63669f09c. Write the dream's
  `var h: [0..m]?Int = None` and `h.map(v => v ?? rng.int(1, 2))`. Route: the author.
- `port.cp:30-64`: `fn try` and the inline checks add to the captured `found`, where the dream
  has a `candidates` generator and a `count`. The streaming cost of `pairs()` is reported and
  measured, but the generator itself isn't. Either measure `count(... for c in candidates(h))`
  against the closure at case 08 and report the gap, or take the dream's form. Route: the author.

## Language gaps

Reported: `pairs()` materialising (L0543 is kept on purpose), the `1..n` bound at
`count_triples`, the per-candidate list sort (the reason for the arithmetic test), and the
`-DLOCAL` overflow checks under `cpl test`. Two earlier gaps are now marked as no longer
reproducing (the closure-local copy and the `max(by:)!.1` copy).
