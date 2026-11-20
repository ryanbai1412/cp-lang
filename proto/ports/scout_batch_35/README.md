# Scout batch 35: COCI and COI

Ten submissions from `corpus/submissions/` whose problem code starts with
`coci` or `coi`, rethought in CP Lang v2 (`docs/one-page-cheat-sheet.md`).
None is in `proto/ports/`, an earlier scout batch or a `blind_critic_*`
directory (checked against `main` at `c8e381f`). The friction they showed
is in
[`research/friction-backlog-batch-35-coci-coi.md`](../../../research/friction-backlog-batch-35-coci-coi.md)
(items `K1`...).

The brief asked for ten hard problems. The bucket did not have ten hard
ones left: twelve `coci*`/`coi*` problems with an accepted submission were
unported when the batch was picked, and the two not taken are first
problems of a round (coci08c1p1, coci21c1p1). So this batch is the three
hard ones (coci21c1p3, coci21c1p4, coci21c1p5), three of middle weight
(coci21c3p2, coci21c6p2, coi07p1) and four easy ones. After it the bucket
is empty apart from those two.

```sh
python3 proto/ports/scout_batch_35/check.py                 # every port, 200 random inputs each
python3 proto/ports/scout_batch_35/check.py coci21c1p4 1000 # one port, 1000 inputs
python3 proto/ports/scout_batch_35/loc.py                   # the line counts below
```

## What runs

Each problem has two files. `<code>.cp` is the port as the v2 cheat sheet
spells it. Neither prototype accepts all of v2, so these ten files were
not run. `today/<code>.cp` is the same algorithm in the forms the
prototype takes, and that is what `check.py` runs against the original
C++:

- Eight are built to C++17 by `proto/cp.py build` and agree with the
  original on 200 random inputs each (seeded), one in ten medium-sized
  (for example 3000 vertices for coci21c1p3, 3^7 cards for coci21c1p4,
  40 by 40 for coci21c6p2).
- coi07p1 needs floating point and coci21c2p1 string interpolation, which
  only the reference interpreter has. Those two run under
  `proto/cp.py lib-run` on 60 small inputs each.
- coci21c1p5 accepts any largest family of disjoint longest increasing
  subsequences, so its output is judged against the statement; the count
  and the length must equal the original's.
- coi07p1 prints three decimals and the interpreter has no `{t:.3f}`, so
  the runnable form rounds by hand and is compared within 0.0015.

Six of the built forms were also run once at the full limits (`-O2`,
hand-made large inputs, not adversarial ones), first line or whole output
compared with the original:

- coci21c2p2 (1000 boxes, 1000 permutations, 5 * 10^5 queries): 0.01 s
  against the original's 0.60 s; the original builds a 1000 by 1000
  reachability table, the port a `Dsu`.
- coci21c1p5 (10^6 values; random, 1000 disjoint chains, sorted): 0.09,
  0.08 and 0.16 s against 0.16, 0.29 and 0.83 s, and 100 MB against
  464 MB on the sorted input.
- coci21c1p4 (all 3^12 cards, and a random half): 0.04 s, same as the
  original.
- coci21c3p2 (a path of 10^5 vertices): 0.01 s.
- coci21c6p2 (500 by 500): 0.23 to 0.33 s against 0.12 to 0.17 s.
- coci21c1p3 (10^5 vertices; one cycle, and a path with a triangle):
  0.75 s and 185 MB against 0.03 to 0.12 s and 30 MB. The answer is right
  but the memo is hashed (K1 in the backlog). It needs `ulimit -s
  unlimited`, as the original does.

How statement-first this was: not at all. The DMOJ API answered 403 for
every problem, so each statement was reconstructed from the accepted C++
and from memory of the published COCI tasks. For coi07p1 nothing but the
C++ was available: the statement in the port's header is inferred from
the code, and its check only shows the port computes what the original
computes. Four ports use a different algorithm from their original
(coci21c2p2, coci21c1p4, coci21c1p5, coci21c6p1); those were not checked against an independent brute force,
only against the original.

## The ports

Non-blank, non-comment lines: v2 port / original, then the original
without what a perfect C++ library would supply (`<code>.strip`).

- `coci21c1p3.cp` (23 / 177, 115 stripped): Logičari. A graph with n
  vertices and n edges; `Dsu` finds the cycle edge, `Rooted` gives the
  tree, and a `?Int` `@memo` does the DP for each guess of the two ends.
- `coci21c1p4.cp` (24 / 239, 76): Set. Counting triples of ternary words
  summing to zero with a Fourier transform over Z_3^k, in numbers
  a + b * omega.
- `coci21c1p5.cp` (34 / 189, 102): Volontiranje. Levels of the longest
  increasing subsequence, then chains grown with one stack. The original
  uses a segment tree per level.
- `coci21c3p2.cp` (20 / 80, 46): Cijanobakterije. Sum of tree diameters
  in a forest.
- `coci21c6p2.cp` (19 / 71, 41): Zemljište. Row bands and a two-pointer
  over column sums.
- `coi07p1.cp` (12 / 59, 28): a left-to-right sweep in `F64`.
- `coci21c2p2.cp` (9 / 82, 48): Kutije. `Dsu` over the boxes.
- `coci21c3p1.cp` (9 / 63, 29): Lista. `first` over (start, length).
- `coci21c2p1.cp` (8 / 59, 25): Kaučuk. Section numbering.
- `coci21c6p1.cp` (7 / 62, 32): Med. Rank by tuple order.

In total 165 lines against 1081 raw (85% fewer) and against 542 stripped
(70% fewer). The runnable `today/` forms are 250 lines (77% and 54%).
