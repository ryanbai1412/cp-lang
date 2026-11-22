# Scout batch 7

Ten hard data-structure and offline-query submissions from the corpus,
re-solved on top of `stdlib/`. This is a Stage-1 critique pass: nothing in
`crates/` or `proto/cpc/` was changed, and what the ports taught us is in
[`research/friction-backlog-batch-7.md`](../../../research/friction-backlog-batch-7.md).

Like batch 2, these files run. They are written for the reference
interpreter (`proto/cpi/`), and each one is checked against its original:

```text
$ python3 proto/ports/scout_batch_7/check.py                # every port, 60 random inputs each
$ python3 proto/ports/scout_batch_7/check.py noi04p1 200    # one port, 200 inputs
$ python3 proto/cp.py lib-run proto/ports/scout_batch_7/noi04p1.cp < input
```

`check.py` compiles the original from `corpus/submissions/`, feeds both
sides the same small random inputs and compares the output. All ten agree
on 200 inputs each.

What that does and does not show. The inputs are small (a few dozen
commands, up to about 12 elements), because the interpreter is slow, so
the ports are checked for answers and not for speed or memory. The
interpreter does no static checking, so nothing here has been through the
type checker or the borrow rules. Three inputs are narrower than the
judge's, and the backlog says why for each:

- `noi03p2`: the inserted text has no spaces, because the interpreter
  reads whitespace-separated tokens only.
- `ioi12p3`: the original is a set of functions for the IOI grader, so
  `check.py` adds a `main` and the line format is ours ("T c", "U x",
  "P i" after a count), not a judge's.
- `cco17p3`: the original is only run on coordinates up to 10^12. Above
  that it reads freed memory (it reserves 110 trie nodes per stroke and a
  stroke near 10^18 can need 119), so on those inputs the port is compared
  with a brute force written from the statement, which `check.py` also
  holds the original to wherever the original is defined.

Where the interpreter lacks a construct the language has (`enum` and
`match`, `else if`, an open slice `xs[i..]`, `enumerate`), the port uses
the nearest spelling that runs. Those lines are listed under "Gaps in the
reference interpreter" in the backlog, so they are not mistaken for the
intended style.

## The ports

The originals are in `corpus/submissions/`.

- `ioi01p1.cp` from `3477640.cpp`: point add and rectangle sum on a grid,
  as a Fenwick tree whose slots are `Fenwick<Y, Int>` trees.
- `noi01p1.cp` from `6454234.cpp`: a union-find that keeps every animal's
  kind relative to its parent, modulo 3.
- `noi04p1.cp` from `3738402.cpp`: the k-th highest salary under global
  raises and cuts. The original uses an order-statistics tree; the port
  reads the commands twice, to number the salaries and then to count them
  in a Fenwick tree.
- `noi03p2.cp` from `5032747.cpp`: a text editor with a cursor. The
  original uses a rope; with at most 4000 edits a piece table (a list of
  strings, cut where an edit lands) is enough.
- `ccc14s4hard.cp` from `5672273.cpp`: the area under panes of glass
  whose tints add up to a threshold. A sweep over x, and square-root
  blocks over the strips between y coordinates.
- `tsoc15c2p6.cp` from `3657404.cpp`: range add and range minimum on a
  lazy tree, 11 lines.
- `cco17p3.cp` from `4843963.cpp`: a binary trie over x in which every
  node owns a trie over y, built only where a stroke went. Nodes live in
  one list and are named by position.
- `ioi12p3.cp` from `4682653.cpp`: typing with undo of undo. The original
  is a persistent segment tree; the port is the tree of texts, with one
  long jump per node for the ancestor lookup.
- `coci20c6p5.cp` from `4188128.cpp`: the h-index of a subarray, by Mo's
  algorithm. The original moves a Fenwick tree and binary-searches every
  answer; the port keeps h itself, which moves by one per step.
- `sac22cc5p4.cp` from `4575559.cpp`: whether a range of integers is
  covered by a set of intervals. Merging the intervals into runs replaces
  the original's coordinate compression and difference array.

One submission was picked and dropped. `jloi16p5` (`5445696.cpp`, nested
circles by a sweep with an ordered map) is accepted on the judge but
disagrees with its statement when two circles sit side by side inside a
third: for the circles `0 0 10`, `0 -5 2` and `0 5 2` it prints 100, and
the area covered an odd number of times is 92. A port could not be checked
against it, so `sac22cc5p4` took its place.

## Line counts

Counted the way `proto/loc.py` counts: blank and comment-only lines never
count. "Stripped" removes from the original what a perfect C++ contest
library would have supplied: the template header, the driver, and here the
two-dimensional Fenwick functions, the weighted union-find, the lazy
segment tree, the node allocator, the sort-and-unique helpers, and the
Fenwick tree and query order of the Mo's solution.

- ioi01p1: 27 lines against 66 (59% fewer), 25 stripped (8% more).
- noi01p1: 23 against 93 (75% fewer), 32 stripped (28% fewer).
- noi04p1: 42 against 71 (41% fewer), 34 stripped (24% more).
- noi03p2: 39 against 78 (50% fewer), 38 stripped (3% more).
- ccc14s4hard: 47 against 152 (69% fewer), 116 stripped (59% fewer).
- tsoc15c2p6: 11 against 60 (82% fewer), 17 stripped (35% fewer).
- cco17p3: 39 against 131 (70% fewer), 76 stripped (49% fewer).
- ioi12p3: 29 against 77 (62% fewer), 52 stripped (44% fewer).
- coci20c6p5: 44 against 139 (68% fewer), 69 stripped (36% fewer).
- sac22cc5p4: 18 against 88 (80% fewer), 56 stripped (68% fewer).

Together: 319 lines against 955 (67% fewer) and 515 stripped (38% fewer).
Three ports are longer than the stripped original, and each is one missing
library piece: a Fenwick tree over two domains (ioi01p1), an ordered bag
with ranks (noi04p1) and a sequence that splices (noi03p2). They are the
first three items of the backlog.
