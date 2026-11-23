# Scout batch 20: statement-first DP, greedy, combinatorics and math

Ten hard DMOJ problems solved from the problem statement, before the
accepted C++ in `corpus/submissions/` was opened. The C++ was read only
afterwards, to confirm the input format and to run differential tests.
`dmoj.ca` problem pages sit behind a browser challenge, so "the statement"
here is the DMOJ API metadata (name, limits, types) plus the well-known
contest task.

The findings are in
[`research/archive/frictions/friction-backlog-batch-20-statement-first.md`](../../../research/archive/frictions/friction-backlog-batch-20-statement-first.md)
under item numbers `W1` through `W16`.

## The ports

- `btoi19p2.cp`: Baltic OI 2019 Nautilus, bit-parallel reachability on a grid (`4456161.cpp`).
- `btoi19p4.cp`: Baltic OI 2019 Tom's Kitchen, one knapsack over hired hours (`4456166.cpp`).
- `cco17p4.cp`: CCO 2017 Rainfall Storage, subset sums over an insertion order (`4575277.cpp`).
- `ccc19s4.cp`: CCC 2019 S4 Tourism, block DP with two suffix maxima (`6032792.cpp`).
- `cco23p1.cp`: CCO 2023 Binaria, residue chains and one binomial (`5579927.cpp`).
- `btoi17p5.cp`: Baltic OI 2017 Plus Minus, inclusion-exclusion over alternating rows and columns (`4456190.cpp`).
- `noi02p4.cp`: NOI 2002 Island of Cavemen, one linear congruence per pair (`3741279.cpp`).
- `ceoi18p5.cp`: CEOI 2018 Toys, ordered factorisations over the divisor list (`5323876.cpp`).
- `ioi19p1.cp`: IOI 2019 Arranging Shoes, greedy pairing and a sum tree (`5661847.cpp`).
- `ioi09p8.cp`: IOI 2009 Salesman, two max-trees and a two-way sweep per day (`4811954.cpp`).

Three further problems (`apio07p2`, `ioi07p4`, `dpz`) were solved for this
batch and dropped before the push, because sibling batches had ported them
in the meantime. The backlog still uses what their drafts showed.

All ten build with `proto/cp.py build` and `g++ -O2 -std=c++17`. None of them
imports `std.*`: the three that wanted to (`cco17p4`, `noi02p4`, `ceoi18p5`)
fall back to bare core, which is item `W9` of the backlog.

## Checking

```text
$ python3 proto/ports/scout_batch_20/check.py            # every port, 300 inputs each
$ python3 proto/ports/scout_batch_20/check.py dpz 50     # one port, 50 inputs
```

`check.py` builds a temporary copy of each port, compares it with the
original on 300 small random inputs and then on one largest-size input, and
prints both running times. All ten agree. `btoi17p5` is compared with an
exhaustive count on the small inputs instead of the original, because the
accepted original is wrong on some of them (backlog item `W1`).

Two ports also carry a slow reference marked `@brute`, checked by
`proto/cp.py fntest <port> 300 6` on a temporary copy: `ccc19s4` and
`cco17p4`. The other eight have no in-file reference; for five of them the
valid inputs cannot be described by types (backlog items `W4` and `W8`).

## Size (non-blank, non-comment lines)

The ten ports total 349 lines against 1,217 for the originals, templates
included. Per problem, port then original: `btoi19p2` 44 / 100, `btoi19p4`
24 / 108, `cco17p4` 46 / 85, `ccc19s4` 47 / 213, `cco23p1` 27 / 174,
`btoi17p5` 27 / 85, `noi02p4` 37 / 104, `ceoi18p5` 27 / 110, `ioi19p1`
27 / 105, `ioi09p8` 43 / 133. The port counts include the `@brute`
references and the inlined library fallbacks. A library-stripped count was
not made.
