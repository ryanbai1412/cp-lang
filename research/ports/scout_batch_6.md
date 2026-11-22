# Scout batch 6

Ten greedy, sorting, two-pointer, sweep-line and coordinate-compression
submissions from the corpus, re-solved in CP Lang. This is a Stage-1
critique pass: nothing in `proto/cpc/` or `crates/` was changed for it, and
what the ports taught us is in
[`research/friction-backlog-batch-6.md`](../../../research/friction-backlog-batch-6.md).

Each problem is here twice.

- `<name>.cp` is the solution as we would like to write it. A line that
  the prototype rejects today is marked `// WISH Sn`, and `Sn` is an item
  in the backlog (`F`, `G`, `B`, `D` and `H` numbers are items of earlier
  batches). These files do not build.
- `today/<name>.cp` is the same solution in the form that builds with
  `proto/cpc` now. Every one of them agrees with its original.

```text
$ proto/ports/scout_batch_6/today/stress.sh          # 200 random cases each
ccc21s3: stress OK: 200 tests agree (0 rejected by where)
...
$ proto/ports/scout_batch_6/today/stress.sh 150 25   # 150 cases, larger inputs
```

`stress.sh` uses `proto/cp.py stress`, so the inputs come from the types in
each port. The one exception is `cco04p2`, whose input is words like `3-5`
that the typed generator cannot make (backlog item S8); `cco04p2_gen.py`
makes its inputs. Each port was also run once on a hand-made input at the
problem's largest size and compared with its original: all ten agree, and
the slowest takes 0.17 s.

`proto/run_all.sh` only looks at `proto/ports/*.cp` and
`proto/ports/hillclimb_dp_graph/`, so this folder does not affect it.

The ports, with code lines (no blanks or comments) of the wanted form and
of today's form against the original's lines from its first declaration
on:

- `ccc21s3` (10, 14 against 35): Lunch Concert. The first range end where
  the slope of a convex cost stops being negative.
- `coci11c4p4` (12, 16 against 54): Ograda. Sort the positions by how much
  they gain from a tall board, and hand the boards out in order.
- `cco17p5` (9, 10 against 61): Professional Network. One heap of the
  people still waiting, walked from the most demanding down.
- `nccc8s3` (17, 17 against 50): the smallest subsequence that keeps each
  number once. A stack that pushes out larger numbers that come again.
- `dmopc20c4p3` (11, 21 against 27): Roving Roombas. A sweep down the
  heights that counts the columns at or before a point.
- `joi16fp2` (18, 33 against 61): Semiexpress. The k - n largest gains,
  with no heap.
- `dmopc21c8p1` (14, 21 against 35): Peanut Planning. A sorted list given
  out from the front and promoted from the back into a stack.
- `cco04p2` (11, 27 against 51): tiles into the fewest rows. Patience
  sorting with a binary search over the rows.
- `etsr23r1p2` (19, 35 against 33): a set of ranges that throws out what a
  new range touches.
- `dmopc20c7p1` (20, 22 against 61): Bob and Balance. Pair the two most
  frequent kinds, again and again.

Six of them changed shape rather than being translated:

- `cco17p5`: the original keeps two heaps, a `used` array and refunds a
  price when a paid friend would have come for free. The port walks the
  demands downwards and pays the cheapest person waiting whenever the
  others are too few. Checked on 1,700 random cases as well.
- `nccc8s3`: the original sweeps the last occurrences with a `set` and two
  pointers. The port is the usual monotone stack.
- `coci11c4p4`: the original fills valleys, peaks, the two ends and the
  slopes in four passes with two pointers. The port gives every position
  one sort key.
- `joi16fp2`: the original pushes the next gain of a stretch into a heap
  each time it pops one. The gains of a stretch never grow, so the port
  lists them all and takes the largest.
- `dmopc21c8p1`: the original erases from a `multiset` by `lower_bound`.
  The smallest number left only grows, so the port needs no search.
- `dmopc20c4p3`: the original erases climbers from an order-statistics
  tree. The port goes the other way down the heights and only adds.

`etsr23r1p2` is the one port whose today form is longer than its original:
the prototype's `Set` and `Map` cannot be asked for the first key at or
after a value, so the port keeps its ranges in a max-tree over compressed
coordinates. That is backlog item S13.

Two problems were ported and then dropped because batches 2 and 7 landed
them first (`cco20p2`, `sac22cc5p4`). What they showed is in the backlog
under S11, S14 and S18.
