# Scout batch 29: number theory, linear algebra and contest classics

Ten submissions from `corpus/submissions/`, rethought in CP Lang. None is
in `proto/ports/`, an earlier scout batch or a `blind_critic_*` directory.
The friction they showed is in
[`research/friction-backlog-batch-29-nt-linalg.md`](../../../research/friction-backlog-batch-29-nt-linalg.md)
(items `O1`...).

A search of the unported accepted submissions found no Möbius or
divisor-sum sieve, no CRT, and no Gaussian elimination over GF(2) or
floats; earlier batches took them. What was left in this area is here: a
determinant modulo a prime, a sieve with inclusion-exclusion, exact
fractions and squares one factor past 64 bits, gcd/lcm counting, a signed
knapsack on a bitset, a product-and-maximum segment tree, a lazy segment
tree of regression sums, and two sorting classics with exact ratios.

```sh
python3 proto/ports/scout_batch_29/check.py               # every port, 60 random inputs each
python3 proto/ports/scout_batch_29/check.py noip20p1 200  # one port, 200 inputs
python3 proto/cp.py test proto/ports/scout_batch_29       # the test of wide.cp
python3 proto/cp.py lib-run proto/ports/scout_batch_29/ioi15p4.cp < input
```

Every port agrees with its original on 60 small random inputs
(`check.py`, seed 29; 15 for noimps22p4, whose tables take the interpreter
two seconds to build). The inputs are small, so the check is about
answers, not speed; nothing was run at the real limits, and the ports
were only run on the reference interpreter, not compiled by `proto/cpc`.
ioi09p2 and noi20p4 accept any best answer, so both outputs are judged
against the statement; sdoi17r1p6 prints decimals, compared to within
2e-5. The original of ioi15p4 is three grader functions; `check.py`
appends a `main`.

The ports were not written statement-first. For each problem the accepted
C++ was read before the port was written, and the port then restructured
around the idea rather than the code. coci25c3p5 is the one port whose
statement was not known at all: it is the formula of the original.

What each port does differently from its original:

- **shoi16p4** (matrix-tree determinant, inclusion-exclusion over
  companies): the Laplacian is built directly over `domain Rest`, the
  cities without the last one, instead of building n by n and deleting a
  row and a column; the determinant tracks the sign of row swaps, which
  the original omits.
- **noimps22p4** (sieve, 13 small primes as a mask, one big prime per
  value): the original runs a two-state parity DP over the big query
  primes; the port multiplies `2^c - 1` per big prime, which is the same
  sum in closed form, and walks only the submasks of the query's small
  primes.
- **noip20p1** (water split over a DAG, exact fractions up to 60^11): the
  original keeps reduced `__int128` fractions and takes a gcd per
  addition. Every denominator is 2^a 3^b 5^c, so the port keeps a wide
  numerator over three exponents and never needs a gcd or a wide
  division.
- **noip19p5** (monotone-queue partition, n up to 4e7 from a generator,
  answer up to 1.6e33): the cuts are `Bounds<Pos>`, the queue holds cuts,
  and the answer is summed in `Wide`.
- **noip19p4** (count meal plans with no ingredient in more than half):
  the "lead" of the fixed ingredient is `domain Lead = -n..=n`, so there
  is no `+ n` offset anywhere.
- **ioi15p4** (Horses): the original keeps a `std::set` of the years with
  x > 1; the port asks a max segment tree for the previous such year with
  `min_left`, so there are three trees and no set.
- **ioi09p2** (Hiring): the original uses two Fenwick trees over
  qualification ranks and a binary search per candidate; the port keeps
  the hires in a max-heap and drops the most qualified while over budget.
- **noi20p4** (dishes of two ingredients): the subset sum has negative
  weights; the port counts from "every negative item taken", so every
  step shifts the bitset up. The greedy scans for the scarcest and the
  most plentiful ingredient instead of keeping an ordered set.
- **sdoi17r1p6** (range add, range "set to S + i", regression slope): the
  stdlib `LazySegTree` with a six-field value and a three-field edit; the
  node keeps where it starts because the set depends on the position.
- **coci25c3p5**: gcd and lcm arithmetic, one line of output.

`wide.cp` is a non-negative number below 1e36 in two base-1e18 halves,
with only what noip20p1 and noip19p5 need. It exists because the
interpreter has no `I128` (O1).

Known gaps against the originals: noip19p5 at n = 4e7 and sdoi17r1p6 with
sums past 9.2e18 were not run; in sdoi17r1p6 the last multiplication
(`num * 1e6`) is done in `Int` and traps where the original widens to
`__int128` (O2). ioi09p2's original hires one candidate even when nobody
is affordable; the port prints 0 there, and the generator keeps one
candidate affordable.
