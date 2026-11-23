# Scout batch 5

Ten flow, matching, greedy, binary-search, geometry and interactive
submissions from the corpus, re-solved the way we would like to write
them. This is a Stage-1 critique pass, in the form of
[`scout_batch_3/`](../scout_batch_3/): the files in this folder are design
sketches, and every line the prototype rejects today is marked
`// WISH Kn`, an item in
[`research/archive/frictions/friction-backlog-batch-5.md`](../../../research/archive/frictions/friction-backlog-batch-5.md).
A marker with another letter is an item of an earlier batch that came up
again (`F` batch 1, `G` batch 3, `B6` batch 4). A bare `Kn` without `WISH`
marks a line that is accepted today and misbehaves. (`H` is batch 8's
letter.)

None of the ten sketches builds yet. [`today/`](today/) holds nine of them
in the form a prototype accepts now, and those were run against their
originals. `proto/cpc/` and `crates/` were not touched, and
`proto/run_all.sh` only looks at `proto/ports/*.cp`, so nothing here
affects it.

```
$ proto/ports/scout_batch_5/check.sh        # 100 inputs or conversations each, about 40 s
noip10p3: stress OK: 100 tests agree (0 rejected by where)
...
```

## The ten

Each line names the original in `corpus/submissions/`, the idea, and how
the `today/` form was checked. The runs listed here were made at commit
`f4f53ad`; after rebasing onto `de5b419`, `check.sh 100` passes again.

- `noi06p4.cp` from `5032812.cpp` (flow): stations cost money, a user pays
  when both of theirs are built; a maximum-weight closure. Today: runs on
  the reference interpreter with `std.flow`; agrees on 150 script-made
  inputs of up to 7 stations and 8 users.
- `cco23p6.cp` from `5618725.cpp` (matching): pairs of equal sticks matched
  with odd sticks by a monoid on a segment tree. Today: stress OK, sizes 6
  and 40.
- `noip10p3.cp` from `5205348.cpp` (binary search): the mildest anger x such
  that the feuds above x are 2-colourable. Today: stress OK, sizes 6 and 40.
- `cco22p1.cp` from `4801881.cpp` (binary search): furthest end from every
  start for which the "shorter than" claims have no cycle. Today: stress
  OK, sizes 6 and 12; 83 to 88 % of generated inputs are rejected (K19).
- `ioi13p5.cp` from `4688418.cpp` (greedy inside a binary search): a
  grader-style original, so `check.sh` appends a `main`. Today: stress OK,
  sizes 5 and 30.
- `btoi17p1.cp` from `4456179.cpp` (greedy): peel the person of least
  degree, then try every subset of at most k - 1 neighbours. Today: built;
  agrees on 300 script-made symmetric inputs of up to 14 people, since
  the typed generator cannot make them (K4).
- `ioi06p2.cp` from `5034372.cpp` (rectangles on a grid): 2-D prefix sums
  and a sliding minimum in both directions. Today: stress OK, sizes 4, 6
  and 9. At `f4f53ad` it needed a `where` that rejected 86 to 93 % of
  the generated inputs (K3); `722c0ea` fixed that and the `where` is gone.
- `cco18p4.cp` from `4801866.cpp` (interactive): least total distance to
  hidden tokens within 170 questions. Today: `proto/interact.py` with
  `today/cco18p4_judge.py`; 200 conversations of size 6 and 60 on a
  10^7 x 10^7 grid accepted for the port and the original.
- `yac6p3.cp` from `5149597.cpp` (interactive geometry): two hidden cells
  from four corner distances and one probe. Today: `proto/interact.py`
  with `today/yac6p3_judge.py`; 300 conversations of size 5 and 100 of
  size 1000 accepted for both.
- `etsr23r3p2.cp` from `5529780.cpp` (geometry): layers of a folded sheet
  under a point, by unfolding. No `today/` form: every coordinate is a
  float (F15).

What the checks do not show: sizes are small, so nothing here was timed or
measured for memory, and `noi06p4` ran on the interpreter, which does no
static checking. The judges were written for this batch from the
originals' behaviour, not from the official statements: the limits of 170
and 5 questions are the original's own assertion and the number it uses.
Input bounds in the declarations are from memory of the statements where
the original does not state them.

## Shape changes to look at first

- `cco18p4.cp`: the original is a golden-section search with floats; the
  port is a binary search on the slope, 2 questions a step (at most 97 in
  all for 10^7 x 10^7).
- `cco22p1.cp`: Kahn's algorithm (`topo`) instead of the original's
  recursive colouring.
- `ioi13p5.cp`: `first_where` returns `none` when even t minutes fail, which
  replaces the original's separate "some toy fits no robot" test.
- `noi06p4.cp` (sketch only): written as a closure problem; the `today/`
  form builds the same network as the original.

## Size

Code lines (no blanks or comments) as sketch / today / original from its
first declaration on, which leaves out the include and debug template
(the library-stripped measure was not done in this pass):
noi06p4 19 / 19 / 94, cco23p6 20 / 28 / 75, noip10p3 9 / 25 / 64,
cco22p1 16 / 35 / 72, ioi13p5 17 / 29 / 50, btoi17p1 13 / 23 / 89,
ioi06p2 17 / 29 / 95, cco18p4 10 / 21 / 69, yac6p3 17 / 18 / 36,
etsr23r3p2 12 / none / 44.
