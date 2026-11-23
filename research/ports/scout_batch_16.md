# Scout batch 16: messy fallbacks and promises a type cannot hold

Ten submissions from `corpus/submissions/` picked to be awkward on purpose,
for a Stage-1 friction pass over two blind spots:

1. **Bare-core fallback.** Problems where domains, grids, graphs and
   algebraic trees do not fit: text that has to be scanned by hand, exact
   geometry, a randomized fill with restarts, output in the judge's
   own sentences.
2. **Promises a type cannot hold.** Inputs that are only valid as a whole:
   a planar subdivision, walls that do not cross, a square that is promised
   to be completable, a hidden word whose first letter never comes back, a
   wiring that is a bijection.

None of the ten is in `proto/ports/` or an earlier scout batch. The
friction they showed is in
[`research/archive/frictions/friction-backlog-batch-16.md`](../../../research/archive/frictions/friction-backlog-batch-16.md)
(items `M1`... for the bare core, `Q1`... for stress generation).

Seven ports run today on the reference interpreter and are checked against
their originals:

```sh
python3 proto/ports/scout_batch_16/check.py               # every runnable port, 60 random inputs each
python3 proto/ports/scout_batch_16/check.py noi03p1 200   # one port, 200 inputs
python3 proto/cp.py lib-run proto/ports/scout_batch_16/noi03p1.cp < input
```

The inputs are small and every generator in `check.py` is constructive: it
builds an input that keeps the statement's promise instead of drawing one
and throwing it away. Those generators are the Python stand-ins for the
`gen` hooks the backlog asks for (Q1 to Q4), so they are part of the
evidence, not only test plumbing.

The ports, with their non-blank, non-comment lines against the original's
solution lines (the original without its template and debug macros):

- `noip07p2.cp` (18 / 56): string expansion. Hand-scanned text; the three
  output parameters become three steps on one list.
- `ccc00s5.cp` (27 / 67): exact geometry. Each rival sheep cuts the fence
  with one linear bound; the ends stay exact fractions.
- `cco05p4.cp` (20 / 43): the shortest run with a prime sum, printed in
  the judge's sentence. Plain loops; the original's sieve to 10^8 becomes a
  test of the few sums that are asked about.
- `coci21c6p5.cp` (41 / 86): one bracket from each pair, kept balanced. A
  greedy with a heap of pairs that can still be flipped.
- `ccc19s3.cp` (39 / 116): complete a 3 x 3 arithmetic square. Randomized,
  many right answers; the six lines are data, so rows and columns share one
  rule where the original writes it twice.
- `noi03p1.cp` (50 / 111): matchstick equation. A hand-written scanner,
  then a search over one-match moves.
- `ioi00p4.cp` (32 / 67): regions of a planar map, joined by shared walls,
  then all-pairs distances.

Three are ideal sketches that do not run. Each says so at its top.

- `ioi18p1.cp` (26 / 44): interactive, a hidden word. The solution is
  complete; the `protocol` and `@interactor` are what `cpl stress` needs,
  and the judge's hidden word has a rule only its generator can keep (Q5).
- `ioi13p4.cp` (29 / 58): interactive, switches wired to doors. The hidden
  wiring is a bijection (Q5).
- `ioi07p2.cp` (52 / 136): walls in the plane that meet only at their ends.
  Faces by union-find over wall sides. Carries the `gen` and `assume` lines
  the backlog proposes (Q1, Q2).

ccc18s3 (RoboThieves) was ported first and then dropped: batch 12 landed
with it while this batch was being written. Three more candidates were
read and left out: noip08p4 (twin-stack sorting) and
dmpg17g3 (bracket recolouring) because an honest rethink of either needs
more than a scout pass, and ioi11p6 (parrots) because its two-phase grader shows nothing the two
interactive sketches do not.
