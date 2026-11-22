# Scout batch 13

Ten binary-search, two-pointer and divide-and-conquer submissions from the
corpus, re-solved in CP Lang. This is a Stage-1 critique pass. Nothing in
`proto/cpc/` or `crates/` was changed for it. What the ports taught us is
in
[`research/friction-backlog-batch-13.md`](../../../research/friction-backlog-batch-13.md).

Each problem has a sketch in this folder: the port as it should read. The
sketches do not build; a `// WISH R3` comment marks a line that needs
backlog item R3 (other letters are items of earlier batches). Seven of
the ten also have a form in `today/` that builds with `proto/cpc` now and
agrees with its original:

```
$ proto/ports/scout_batch_13/check.sh        # every today/ form, 100 inputs each
coci12c1p4: stress OK: 95 tests agree (205 rejected by where)
ioi11p3: stress OK: 100 tests agree (0 rejected by where)
ioi05p4: stress OK: 100 tests agree (0 rejected by where)
tle17c5p6: stress OK: 100 tests agree (0 rejected by where)
apio07p2: stress OK: 100 tests agree (0 rejected by where)
coci21c4p4: OK: 100 tests agree (distance, and the towns as a set)
cpc21c1p5: interact OK: 100 conversations accepted for both the port and the original
```

Five are stressed with inputs generated from their own input types.
`coci21c4p4` accepts any k towns, so `check.sh` compares the distance and
the towns as a set. `cpc21c1p5` is interactive: the port and the original
talk to `today/cpc21c1p5_judge.py`. The inputs are small (the sizes are in
`check.sh`); no port was run at the problem's largest size or timed.

Three have no `today/` form. `ceoi17p2` needs real numbers, which
`proto/cpc` does not have; its new algorithm was compared with the
original on 400 random inputs through a Python copy, which is not in the
repository. `btoi18p3` is a three-part interactive problem and was not
built. `ioi14p6` needs two library pieces that do not exist, and its
sketch was not checked in any way.

`proto/run_all.sh` only looks at `proto/ports/*.cp`, so this folder does
not affect it.

## The ports

Each port names its original on its `// original:` line. The three counts
are code lines (no blanks or comments) of the sketch, of the `today/`
form, and of the original from its first declaration on, without its
debug lines.

- `coci12c1p4.cp` (5, 4, 31): Ljubomora. Search for the smallest largest handful.
- `coci21c4p4.cp` (27, 42, 76): Parkovi. Search for a distance on a tree, and report the parks that reach it.
- `cpc21c1p5.cp` (13, 14, 78): hidden rectangles, found one side at a time by four searches (interactive).
- `tle17c5p6.cp` (18, 26, 57): a NAND circuit. Search for the input that flips the output.
- `ioi11p3.cp` (10, 20, 52): Rice Hub. The longest affordable block of fields.
- `ceoi17p2.cp` (14, none, 46): Sure Bet. Two sorted lists walked together.
- `ioi05p4.cp` (8, 17, 62): Birthday. The widest gap between shifts on a circle.
- `apio07p2.cp` (10, 36, 92): Backup. Divide and conquer over convex cost lists.
- `ioi14p6.cp` (14, none, 99): Holiday. A divide-and-conquer DP over the number of days.
- `btoi18p3.cp` (29, none, 224): Worm Worries. A local maximum on a line, a sheet and a box (interactive).

Four of them changed shape rather than being translated:

- `ioi05p4.cp` has no search. The original tries a distance and counts,
  with a difference array on a circle, the turns that suit every child.
  The port sorts the shifts and reads the answer off the widest gap.
- `ceoi17p2.cp` has no search either. The original halves a real number
  200 times and runs a greedy walk for each; the walk alone gives the
  answer.
- `ioi11p3.cp` moves two ends where the original runs a binary search
  from every start.
- `apio07p2.cp` works on the gaps between offices, not on the offices, so
  a run of one gap is the whole base case (the original solves runs of up
  to ten offices with a quadratic DP), and it merges the steps of two
  convex lists instead of sorting them.

Ports of `coci10c7p4` and `dmopc18c1p6` were written for this batch and
dropped before the push, because batches 12 and 14 landed them first.
