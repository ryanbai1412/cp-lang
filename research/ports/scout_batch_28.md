# Scout batch 28: hard dynamic programming

Ten DP submissions from `corpus/submissions/`, rethought in CP Lang. None
is in `proto/ports/`, an earlier scout batch or a `blind_critic_*`
directory
(checked against `main` at `72e01c1`). An eleventh port, coci21c1p2, was
written, passed the same check, and was dropped because batch 23 landed
with it while this batch was in progress. The friction they showed
is in
[`research/archive/frictions/friction-backlog-batch-28-hard-dp.md`](../../../research/archive/frictions/friction-backlog-batch-28-hard-dp.md)
(items `D1`...).

The brief asked for bitmask/SOS, digit, interval, convex-hull and knapsack
DP. The corpus has no unported sum-over-subsets submission and no unported
plain digit DP (dps, ccc04s5 and coci21c1p4 are already ports), so those
two are covered by the nearest things left: a profile DP over subsets of a
column (dwite10c3p3) and a DP over the bits of a 60-bit answer (ioi20p4).

```sh
python3 proto/ports/scout_batch_28/check.py                # every port, 200 random inputs each
python3 proto/ports/scout_batch_28/check.py ioi16p6 1000   # one port, 1000 inputs
```

Every port agrees with its original on 200 random inputs (`check.py`,
seed 28), about one in ten of them medium-sized (for example 14 by 14 for
ioi09p4, 400 points on a 10^6 grid for ioi16p6, 60 kinds for ioi20p4).
The ports are built to C++17 by `proto/cp.py build`, not run on the
interpreter. ioi00p5 accepts any best set of offices, so there both
outputs are judged against the statement. Two originals are IOI functions
without a `main` (ioi20p4, ioi16p6); `check.py` appends one that reads the
sample-grader format named in the port's header.

Each port was also run once at the full limits, with the checked local
build at `-O2`. Eight finish in under 0.5 s on the inputs tried (random
ones at the largest sizes, not built worst cases). The two to know about:

- noip21p3 takes 0.05 s on 10^4 values, but 1.1 to 1.4 s and 210 to
  257 MB on random 400-value inputs spread over 1..=600, over the 1 s limit. The original is fast on
  both because it picks a flat table below n = 400 and a hash table
  above; `@memo` hashed both (D1 in the backlog).
- ioi20p4 was only run on random stock, not on an input built to make
  the memo large.

How statement-first this was: the DMOJ API gave the name, tags and limits
for eight problems and nothing for noip10p2 and noip05p2; the statements
themselves are the published IOI, NOIP, COCI and DWITE tasks, recalled
rather than fetched. While choosing problems, the tail of several
originals (their `main` and DP function) was on screen before the port
was written, so only ioi09p4, dwite10c3p3 and ioi04p2 were
solved before their original was read.

The ports, with their non-blank, non-comment lines against the original's
non-blank lines:

- `ioi09p4.cp` (14 / 78): Raisins, an interval DP over sub-rectangles;
  `@memo` on four domain parameters and a 2D `prefix`.
- `noip08p3.cp` (17 / 66): Passing Notes, two paths through a grid moved
  in step; a `?Int` memo on (steps, row, row).
- `ioi00p5.cp` (26 / 70): Post Office, a partition DP with the offices
  rebuilt from the memo.
- `dwite10c3p3.cp` (19 / 68): 3 by n domino tilings, a profile DP over
  `Subset<Cell>` with `.subsets`.
- `noip10p2.cp` (16 / 67): Tortoise Chess, a dense `@memo` over four card
  counts.
- `noip21p3.cp` (14 / 121): Variance, a knapsack over the running sum
  whose reachable states are dense for small n and sparse for large n.
- `noip05p2.cp` (20 / 60): Crossing the River, a 10^9-long line shrunk to
  the stones before a short-hop DP.
- `ioi04p2.cp` (20 / 69): Hermes, two rolling rows updated in place.
- `ioi20p4.cp` (19 / 57): Packing Biscuits, a hash `@memo` on (bit, limit).
- `ioi16p6.cp` (38 / 160): Aliens, a per-photo fee found by binary search
  and a monotone line container; the original uses a Li Chao tree.
