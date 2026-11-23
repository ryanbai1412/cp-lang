# Scout batch 8

Eleven game-theory, counting, modular-arithmetic, bitset and
meet-in-the-middle submissions from the corpus, re-solved in CP Lang. This is a Stage-1 critique pass. Nothing
in `proto/cpc/` or `crates/` was changed for it. What the ports taught us
is in
[`research/archive/frictions/friction-backlog-batch-8.md`](../../../research/archive/frictions/friction-backlog-batch-8.md).

Every file here runs today and agrees with its original:

```
$ python3 proto/ports/scout_batch_8/check.py          # every port, 200 random inputs each
$ python3 proto/ports/scout_batch_8/check.py dpy 50   # one port, 50 inputs
cco16p4: stress OK: 200 tests agree (0 rejected by where)
...
dpy: interpreter OK: 200 tests agree
```

Eight ports build with `proto/cpc` and are stressed with inputs generated
from their own input types. Three (`dpy`, `ioi04p5`, `yac1p4`) import
`std.mod` or `std.bitset`, which only the reference interpreter
(`proto/cpi`) accepts, so `check.py` feeds them small hand-written random
inputs instead. Those three are checked for answers only: the interpreter
does no static checking and is too slow for full-size inputs. Only
`cco19p3` was also run at full size, by hand: two inputs with 200,000
cities and a hub of 36 branches gave the original's answers.

`proto/run_all.sh` only looks at `proto/ports/*.cp`, so this folder does
not affect it.

## The ports

Each port names its original on its `// original:` line. The counts are
code lines (no blanks or comments) against the original's lines from its
first declaration on.

- `dpk.cp` (7 against 34): Stones. Win/lose table of a subtraction game.
- `dpl.cp` (8 against 26): Deque. Interval game, memoised on the two ends.
- `cco16p4.cp` (17 against 47): O Canada. Grids equal under 2 x 2 flips, counted in pairs.
- `coci21c2p3.cp` (23 against 40): XOR labels on a tree, one labeling per even set of bits.
- `sac22cc4jp4.cp` (21 against 48): a power of a permutation, by walking its cycles.
- `coci25c3p2.cp` (10 against 23): permutations with k cycles, modulo a prime.
- `noip14p6.cp` (23 against 65): roots of a polynomial with 10^4-digit coefficients, modulo two primes.
- `dpy.cp` (17 against 70): Grid 2. Lattice paths around walls with factorial tables.
- `ioi04p5.cp` (23 against 29): Farmer. Greedy, or a subset-sum bitset.
- `yac1p4.cp` (19 against 42): reachable signed totals, a bitset per nesting depth.
- `cco19p3.cp` (48 against 106): Winter Driving. Rerooting on a tree, and a meet-in-the-middle split of up to 36 branches.

Six of them changed shape rather than being translated:

- `cco16p4.cp` does not run the original's elimination. A 2 x 2 flip keeps
  the parity of every row and column and nothing else, so the key of a
  grid is those parities.
- `sac22cc4jp4.cp` never builds a power of the permutation. It walks each
  cycle once and writes the answers by position, O(n) instead of
  O(n log k).
- `coci25c3p2.cp` keeps one row of the recurrence, 5,000 cells instead of
  a 5,000 x 5,000 memo.
- `yac1p4.cp` stores each total with the worth of every letter seen so far
  added to it, so all three moves shift a set upward, and there is no
  two-layer swap.
- `noip14p6.cp` loops over a list of primes instead of instantiating a
  template per modulus.

- `cco19p3.cp` has no recursion and no second search. The original roots
  the tree with two DFS passes; the port scans one top-down order in both
  directions. The original looks up two neighbours of the target for each
  half-sum; the port asks only for the largest total at most half, which a
  single sweep over the two sorted halves answers.

`cco19p3` is one more than the ten this batch set out to port. It is the
only meet-in-the-middle solution that a search of the unported accepted
submissions turned up.
