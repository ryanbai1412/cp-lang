# ioi25_triples

Score: 3/5
Dream: 3/5

Scored cold on main against the statement (IOI 2025 Day 1, Triple Peaks), `dream.cp` and
`port.cp`; the friction report was read only after scoring, to sort gaps into reported and
unreported. The dream is under 4, so it goes back and the port score is provisional.

## Dream failures

The dream is the port: `dream.cp:10-67` is `port.cp:27-84` line for line, workarounds included
(the index loops, the sentinel and the hand argmax that the friction report measured). A dream
should say how the solution ought to read, not what compiles fast today.

- `dream.cp:17-19`, `fn try` adds to the captured `found`, and every candidate site calls it
  for its effect: a hidden side effect (idiomatic-code/R7, caps the dream at 3). Rewrite: a
  `fn candidates(h) -> yields (i, j, k, top)` that yields lines 24-30 and 33-40's triples,
  then `count(ok(c) for c in candidates(h))`.
- `dream.cp:36-37`, `for a in 0..group.len: for b in a + 1..group.len` is index bookkeeping;
  the dream line is `for (i, j) in group.pairs(): crossed(i, j, i + h[j])`. `pairs()` being
  slow (friction "`for (i, j) in group.pairs()` builds every pair first") belongs to the port.
- `dream.cp:56, 59`, `var h: [0..m]Int = m` uses `m` as "no peak yet" (R5); the dream line is
  `var h: [0..m]?Int = None` and `h.map(v => v ?? rng.int(1, 2))`, as friction "Mapping a
  closure's local table copies it" itself says.
- `dream.cp:62-67`, the `var (best, most)` loop is the hand argmax; the dream line is
  `(0..8e5 / m).map(attempt).max(by: count_triples)!`.
- `dream.cp:14`, `h: List<Int>` with positions as raw `Int` throughout (R6). The dream should
  read `h: [Peak]1..n` with `domain Peak = 0..n` and test candidates with `i in Peak`.
- `dream.cp:55`, `2 * size / 3, 4 * size / 3` and `isqrt(6 * m)` are not derived; the header
  should say why |S| near √(6M) gives about M^1.5 triples and why the size is jittered.

## Checklist failures (port)

- `port.cp:31, 89-90`: `h` is read as `[n]1..n` and then mapped to `List<Int>` so
  `construct_range`'s plain list can share `count_triples`; the statement's `1..=N - 1` bound
  is dropped and positions cross as raw `Int` (R5, R6). Not in the friction report: add it
  (route: idioms author, then stdlib for a `count_triples<P>(h: [P]1..n)` that both callers
  can use).
- `port.cp:54`: `cpl lint` warns L0543 (`group.pairs()`). The friction report measures why
  the port keeps the index loops (0.20 s against 0.29 s at `--max`), so this is reported, not
  owed; the lint and `pairs()` codegen are the fix (codegen).

## Owed rewrites

None the port can take today without the measured slowdowns: `pairs()`, the `?Int` map and
`max(by:)!.1` each cost a copy or a materialised list (friction sections 3-5).

## Language gaps

Reported: `Bag`/`sorted` on tuples/`max_by` (section 1), `@grader` signatures (section 2),
`pairs()` materialising (section 3), closure-local map copy (section 4), `max(by:)!.1` copy
(section 5). Missing: the dropped `1..n` element type above.
