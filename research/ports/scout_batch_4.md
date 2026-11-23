# Scout batch 4

Ten dynamic-programming submissions from the corpus, re-solved in CP Lang:
subset, digit, interval, knapsack, permutation, connected-component and
tree DP. This is a Stage-1 critique pass. Nothing in `proto/cpc/` or
`crates/` was changed for it.

Unlike batch 1, every file here builds with today's prototype and agrees
with its original. Where the prototype forced an awkward line, the port
keeps that line, and
[`research/archive/frictions/friction-backlog-batch-4.md`](../../../research/archive/frictions/friction-backlog-batch-4.md)
quotes it next to the line we would rather write.

Each port names its original on the `// original:` line of its header.
To stress all ten against those originals:

```
$ proto/ports/scout_batch_4/stress.sh        # 200 random cases each
dpm: stress OK: 200 tests agree (0 rejected by where)
...
```

`proto/run_all.sh` only looks at `proto/ports/*.cp`, so this folder does
not affect it.

The ports, with code lines (no blanks or comments) against the original's
lines from its first declaration on, which leaves out the include and debug
template:

- `dpu.cp` (14 against 44): Grouping. Best partition of a set, over every subset of every subset.
- `dpo.cp` (14 against 38): Matching. Count perfect matchings over subsets of one side.
- `dps.cp` (16 against 44): Digit Sum. Digit DP on a 10^4-digit bound, without a "tight" dimension.
- `dpn.cp` (10 against 32): Slimes. Interval DP, memoised over pairs of cuts.
- `dpx.cp` (8 against 34): Tower. Exchange-argument sort, then a knapsack over weight.
- `dpm.cp` (17 against 42): Candies. Bounded knapsack count with a sliding window.
- `dpt.cp` (23 against 47): Permutation. Insertion DP over ranks with running sums.
- `joi16op3.cp` (27 against 62): Skyscraper. Connected-component DP with a cost budget.
- `ioi06p4.cp` (36 against 73): The Valley of Mexico. Circular interval DP with the route rebuilt.
- `noi02p3.cp` (37 against 68): Greedy Nine-Headed Dragon. Knapsack merged up a tree.

Three of them (`dps`, `dpx`, `joi16op3`) index their tables by plain ranges
such as `[0..=n, 0..=limit, 0..=2]Z` and not by named domains, because the
indices are amounts that the recurrence does arithmetic on. Item B1 in the
backlog is about when to pick which.

Four of them changed shape rather than being translated:

- `dps.cp` keeps one row for the numbers already below the bound and a
  single value for the prefix still tied with it, instead of a memo over
  (position, tight, remainder).
- `dpn.cp` indexes by the cuts between slimes, so a stretch is `l..r` and a
  split is one cut, with no `k + 1`.
- `ioi06p4.cp` has one memoised function that returns the next city. The
  route is rebuilt by calling it again, so there is no second table of
  choices. It tries the two moves in the original's order of preference,
  which is why the outputs match exactly although the problem accepts any
  route.
- `noi02p3.cp` folds each finished child into its parent along the reversed
  preorder instead of recursing, and treats "which kind of head eats this
  fruit" as a small domain.

Each port was also run once on a hand-made input at the problem's largest
size and compared with its original; all ten agree there, and the slowest
takes 0.09 s.

One problem was dropped: `dpj` (Sushi, an expected value) needs floats,
which the prototype does not have. It is item B5 in the backlog.
