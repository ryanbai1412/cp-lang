# Scout batch 33: hard CCC / CCO / COCI / IOI classics

Ten submissions from `corpus/submissions/`, rethought in CP Lang against
the locked v2 spec (`docs/one-page-cheat-sheet.md`, `3723dca`): DP, trees,
graphs and data structures mixed. None is in `proto/ports/`, scout batches
1 to 32 and 35 or a `blind_critic_*` directory (checked against `main` at
`db2f45d`). Three more ports, ceoi22p5, coci21c1p3 and cco22p3, were
written and passed the same check, and were dropped because batches 32,
35 and 30 landed with them while this batch was in progress; ioi09p5,
cco05p6 and btoi16p1 took their place, which makes the batch easier than
planned. One overlap remains: batch 37 landed with ioi09p5 minutes
before this batch was pushed, so that problem now has two ports. The friction they showed is in
[`research/archive/frictions/friction-backlog-batch-33-classics.md`](../../../research/archive/frictions/friction-backlog-batch-33-classics.md)
(items `C1`...).

```sh
python3 proto/ports/scout_batch_33/check.py                # every port, 40 random inputs each
python3 proto/ports/scout_batch_33/check.py ioi12p6 200    # one port, 200 inputs
```

Each file is the runnable form: it runs on the reference interpreter
(`python3 proto/cp.py lib-run <port>.cp < input`), and a line the v2 sheet
would let us write better is marked `// WISH Cn`. Every port agrees with
its original on 200 random inputs (`check.py`, seed 33), half of them
small (under ten elements) and half medium (up to 150 seats for ccc20s4,
120 operations for btoi15p2, 60 knights for ioi12p6, 40 cars for
ioi09p5). Outputs are compared token by token. cco21p2 and cco20p4
accept many answers; the ports visit things in the original's order, so
their output is equal too and no judge was needed.

Not done: nothing here was built by `proto/cpc` or run at full limits, and
the interpreter does no type or borrow checking. The statements were not
fetched; each header describes the task as recalled and as the accepted
original reads it. `proto/cpc/` and `crates/` were not touched.

## The ten

Non-blank, non-comment lines of the port against the original's non-blank
lines, then against the original with its template, debug macros, I/O
setup and library-grade structures removed.

- `ccc20s4.cp` (17 / 162 / 133) from `4324014.cpp`: Swapping Seats.
  Prefix counts over the round table written twice; one formula per
  rotation instead of the original's incremental 3 x 3 matrix.
- `btoi18p2.cp` (23 / 80 / 52) from `4456171.cpp`: DNA. Two pointers with
  a count of unmet requirements.
- `btoi15p2.cp` (24 / 107 / 57) from `5640898.cpp`: Editor. Undo of
  undo by binary lifting over `?Op` links.
- `ioi14p5.cp` (20 / 123 / 22) from `6567598.cpp`: Friend. The
  construction undone backwards, two numbers per person.
- `ioi12p6.cp` (38 / 88 / 54) from `4848321.cpp`: Jousting Tournament.
  The round tree built with a Fenwick descent, then two passes. The
  original is a grader function; `check.py` appends a `main`.
- `cco20p4.cp` (29 / 77 / 50) from `4801875.cpp`: Travelling Salesperson.
  The Euler walk of a one-colour DFS tree, rotated to each city.
- `cco21p2.cp` (31 / 92 / 54) from `4543410.cpp`: Weird Numeral System.
  BFS over quotients with the digit that led to each.

- `ioi09p5.cp` (25 / 77 / 44) from `5106592.cpp`: Garage. A queue of
  cars and a heap of free spaces.
- `cco05p6.cp` (17 / 65 / 33) from `5150823.cpp`: Windows. A stack of
  windows scanned per click, the hit one moved to the top.
- `btoi16p1.cp` (20 / 82 / 50) from `5323961.cpp`: Bosses. `bfs` from
  every root over boss-to-report arcs.

Total: 244 lines of CP Lang for 953 raw and 549 library-stripped lines
of C++: 74% and 56% fewer.
