# Blind critic 2: mid-contest WA debugging and algorithm pivots

Six corpus problems that had no port (plus two that turned out to have one), each written the way a contest goes wrong: a first draft that is too slow
or subtly wrong (`*_draft.cp`, `*_draft2.cp`), then the solution it was debugged or pivoted
into (`<problem>.cp`). The drafts are kept on purpose and are not accepted solutions. The
findings are in [`research/archive/blind-evals/blind-critic-2-debug-pivot.md`](../../archive/blind-evals/blind-critic-2-debug-pivot.md).

- `dpq` (Flowers): an O(n²) DP, pivoted to a max `SegTree` over heights. Original 4131861.
- `noi15p4` (Homer's Epic): k-ary Huffman without zero padding, then with it. Original 5032756.
- `sac22cc2p4`: a right/down grid DP, pivoted to Dijkstra over cells. Original 4121086.
- `ccc02s4` (Bridge Crossing): greedy full groups, then a DP with the wrong tie-break, then the DP with reconstruction. Original 4157526.
- `cco07p2` (Snowflakes): canonical form under rotation only, then with reflection. Original 3742713.
- `sac22cc1p5`: a 1D scan written for k = 1, pivoted to a 2D DP over (position, groups). Original 3992625.

Two more were written the same way before it turned out that `main` already had ports of
them (`proto/ports/dpz.cp`, `proto/ports/dmpg16s5.cp`, neither of which was read). They stay
here for their drafts:

- `dpz` (Frog 3): an O(n²) DP, then a hull with a flipped `bad()`, then the convex hull trick. Original 4133674.
- `dmpg16s5` (Pizza Bag): an O(nk) scan, then a monotone deque with an off-by-one window, then the fix. Original 5323572.

Run from `proto/` (both commands stamp and build next to the port, so use a scratch copy or
clean up afterwards):

```
python3 cp.py stress $PWD/ports/blind_critic_2/dpz.cp ../corpus/submissions/4133674.cpp 400 10
python3 cp.py locate $PWD/ports/blind_critic_2/noi15p4_draft.cp ../corpus/submissions/5032756.cpp
python3 ports/blind_critic_2/selfstress.py ports/blind_critic_2/dpz_draft2.cp 200 10
```

`selfstress.py` compares a port's main solution with its own in-file `@brute(main) fn slow()`
on sizes 1, 2, ..., which `cp.py` cannot do without a reference `.cpp`.
