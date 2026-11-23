# Scout batch 11

Ten number-theory and linear-algebra submissions from the corpus, re-solved
in CP Lang: matrix powers (modular and max-plus), determinants, a linear
recurrence found from small cases, CRT, sieves, a Mobius sum, modular
counting and polynomial interpolation. This is a Stage-1 critique pass.
Nothing in `proto/cpc/` or `crates/` was changed for it. What the ports
taught us is in
[`research/archive/frictions/friction-backlog-batch-11.md`](../../../research/archive/frictions/friction-backlog-batch-11.md).

Every file here runs today and agrees with its original:

```
$ python3 proto/ports/scout_batch_11/check.py             # every port, up to 200 random inputs each
$ python3 proto/ports/scout_batch_11/check.py noi18p4 50  # one port, 50 inputs
canada21p5: interpreter OK: 200 tests agree
coci21c2p4: interpreter OK: 200 tests agree
noi07p5: interpreter OK: 40 tests agree
noi18p4: interpreter OK: 200 tests agree
noi20p1: interpreter OK: 60 tests agree
noi21p2: interpreter OK: 200 tests agree
noip21p1: stress OK: 20 tests agree (0 rejected by where)
sdoi17r1p1: interpreter OK: 200 tests agree
sdoi17r1p3: stress OK: 200 tests agree (0 rejected by where)
shoi16p3: interpreter OK: 200 tests agree
```

Two ports (`noip21p1`, `sdoi17r1p3`) build with `proto/cpc`
and are stressed with inputs generated from their own input types. The
other eight import `stdlib/` or use two-index tables that only the
reference interpreter (`proto/cpi`) accepts, so `check.py` feeds them small
hand-written random inputs. Those eight are checked for answers only: the
interpreter does no static checking and is far too slow for full-size
inputs, so none of them was run at the statement's limits. `noi07p5`,
`noi20p1` and `noip21p1` get fewer inputs because one run takes about a
second. `sdoi17r1p3` reads `m` up to 200 rather than the statement's 2e7 so
that the generated inputs stay small.

`proto/run_all.sh` only looks at `proto/ports/*.cp`, so this folder does
not affect it.

## A port that was dropped

The first version of this batch had `noip21p2` (a carry DP with binomial
weights). Scout batch 14 landed the same problem while this batch was being
written, so it was replaced by `coci21c2p4`. The compiler messages it
produced are still in the backlog, marked as probes.

## What the corpus did not have

The request named XOR bases and NTT. No accepted submission outside the
existing ports uses either (searched for `basis`, `gauss`, `ntt`, `fft` and
primitive roots), so Gaussian elimination is covered by the two
determinant ports and polynomials by `shoi16p3` and the cyclic convolution
of `sdoi17r1p3`.

## The ports

Each port names its original on its `// original:` line. The counts are
code lines (no blanks or comments) against the original's lines from its
first declaration on.

- `canada21p5.cp` (29 against 106): random flips of a binary string. A Markov chain on the number of differing places, raised to the k-th power.
- `noi20p1.cp` (42 against 106): Delicacy. A max-plus matrix over (city, days to wait), with one jump per festival.
- `noi21p2.cp` (44 against 131): signed disjoint paths in a layered graph. A product of layer matrices and its determinant.
- `noi07p5.cp` (37 against 182): spanning trees of a banded graph for n up to 1e15. Kirchhoff's determinant for small n, then `std.spec.guess_at` finds the recurrence. The original's bitmask DP, Berlekamp-Massey and matrix power are gone.
- `noi18p4.cp` (33 against 131): Dragon Slayer. One linear congruence per dragon, folded with `std.math.crt`.
- `noip21p1.cp` (22 against 115): Count Off. A sieve over numbers with a digit 7.
- `sdoi17r1p3.cp` (34 against 114): sequences with a prime and a sum divisible by k. A power under cyclic convolution.
- `sdoi17r1p1.cp` (34 against 198): product of fib(gcd(i, j)). Mobius inversion and quotient blocks.
- `coci21c2p4.cp` (22 against 97): Magneti. Blocks of magnets grow and merge by increasing radius; `Comb` shares out the free cells.
- `shoi16p3.cp` (36 against 202): Class Ranking. The per-course count is brute-forced at small limits and interpolated; the original's `Poly` type is gone.

Two ports changed shape rather than size. `noi07p5` keeps only the
definition (a determinant) and lets the library find the recurrence.
`shoi16p3` replaces symbolic Faulhaber polynomials with interpolation of
the brute-force sum.
