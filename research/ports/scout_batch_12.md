# Scout batch 12

Ten constructive, ad-hoc, grid-simulation and implementation-heavy
submissions from the corpus, re-solved in CP Lang. This is a Stage-1
critique pass. Nothing in `proto/cpc/` or `crates/` was changed for it.
What the ports taught us is in
[`research/archive/frictions/friction-backlog-batch-12.md`](../../../research/archive/frictions/friction-backlog-batch-12.md).

Nine of the ten build today and agree with their originals:

```
$ python3 proto/ports/scout_batch_12/check.py               # every port, 200 inputs each
$ python3 proto/ports/scout_batch_12/check.py noi05p1 50    # one port, 50 inputs
ccc10s4: stress OK: 200 tests agree
...
cco02p3: judge OK: 200 tests agree
```

Each file is the form that builds with `proto/cpc` at `91fb701`. A line
that we would rather write another way carries `// WISH Ln: the line we
want`, and `Ln` is the item in the backlog. `cco19p1.cp` is the one
sketch: it needs `Float`, which the prototype does not have.

`check.py` makes small random inputs by hand, because most of these
inputs have structure a typed generator cannot say yet (a solvable
sudoku, one `S` in a walled grid, pens that share walls). `cco02p3` is
interactive: `check.py` plays the judge for the port and for the
original and compares both maps with its own flood fill. No port was
run at the problem's largest size, so nothing here says they are fast
enough.

`bugs/let_then_push.cp` is the eight-line program behind backlog item
L0. It builds, and reads freed memory when run.

`proto/run_all.sh` only looks at `proto/ports/*.cp`, so this folder does
not affect it.

## The ports

Each port names its original on its `// original:` line. The counts are
code lines (no blanks or comments) against the original's lines from its
first declaration on.

- `ccc18s3.cp` (52 against 128): RoboThieves. Cameras that see along rows
  and columns, conveyors that carry the robot for free, fewest steps to
  every cell.
- `coci10c7p4.cp` (34 against 104): Poštar. The narrowest band of
  heights in which a flood fill over eight neighbours reaches every house.
- `wossoly21s3.cp` (39 against 100): Fleeing Through the School. Four
  shortest walks in a row on a grid with one-way portals.
- `dmopc21c6p3.cp` (24 against 81): An Art Problem. Colours spread from
  many cells at once, the smaller colour winning a tie.
- `noip09p4.cp` (49 against 87): Target Sudoku. Backtracking on the
  blank with the fewest digits left, for the best weighted score.
- `noi05p1.cp` (36 against 129): Gorgeous Waltz. A sliding-window
  maximum along every row or column, restarted at each piece of furniture.
- `ccc10s4.cp` (35 against 88): Animal Farm. Walls shared by polygons
  become edges, and the answer is the cheaper of two spanning trees.
- `ioi05p1.cp` (38 against 94): Garden. Two rectangles with exactly k
  roses each, by a two-pointer scan run on the grid, its mirror image
  and its transpose.
- `cco02p3.cp` (44 against 113): Return to Blind Man's Bluff. An
  interactive depth-first walk of a room the program cannot see.
- `cco19p1.cp` (18 against 99, sketch): Human Error. A game on a board
  of at most 13 cells, memoised on the whole board, with a real-valued
  answer.

## What was not checked

- Full-size inputs and running time, for every port.
- `cco19p1.cp` was never run.
- `ccc10s4`: `check.py` only makes pens that are joined through inner
  walls. Where they are not, the original adds up a forest and calls it
  a tree, so it is not an oracle there.
- The original of `noi05p1` opens `input.txt`; `check.py` compiles a
  copy with that line removed.
