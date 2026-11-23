# Blind critic 4: v2.1 one-page cheat-sheet contest-pressure & pivot audit

Eight genuinely hard, previously unported IOI and DMOPC problems (including two messy ad-hoc / heuristic-and-constructive problems), each drafted blind from `docs/one-page-cheat-sheet.md` alone (`*_draft.cp`, preserved verbatim before inspecting `proto/cpc/`, `research/`, `decisions.md`, or `stdlib/`), then ported into runnable `proto/cpc` forms (`<problem>.cp`) and differentially verified against the original accepted C++ submissions. Full findings and recommendations live in [`research/archive/blind-evals/blind-critic-4-v21-audit.md`](../../archive/blind-evals/blind-critic-4-v21-audit.md).

- `ioi11p5` (Dancing Elephants): sqrt-bucket decomposition with periodic rebuilds and online move queries. Original `3469626.cpp`.
- `ioi22p1` (Fish): sparse 2-state column DP over sorted per-column fish prefix sums (also Drill 1: failing full-pier greedy -> 2-state sparse DP pivot). Original `4785724.cpp`.
- `ioi20p1` (Plants): cyclic topological extraction with range-add/max (`AddMax<V>`) + ordered ready sets + binary lifting (also Drill 2: naive `O(N^2)` -> `O((N+Q) log N)` upgrade). Original `5725518.cpp`.
- `ioi17p6` (Books): messy ad-hoc permutation cycle-hull nested interval expansion. Original `5659304.cpp`.
- `ioi13p6` (Game): 2D dynamic segment tree over 10^9 x 10^9 grid for online GCD queries using typed/parallel arenas. Original `4688844.cpp`.
- `ioi15p6` (Towns): interactive tree hub-radius + Boyer-Moore branch-majority voting with query budget (also Drill 3: authentic Boyer-Moore bucket-index WA caught by differential fuzzing). Original `5723344.cpp`.
- `ioi11p6` (Parrots): two-phase order-invariant combinatorial encoding/decoding (`encode`/`decode` with `m <= 5n`, 256-value byte alphabet). Original `3735613.cpp`.
- `dmopc21c5p5` (Prime Difference Permutation): non-typeable constructive/heuristic optimization (exact `Subset<V>` Held-Karp DP for `n <= 12` + multi-start 2-opt local search for larger `n`). Original `4223999.cpp`.

`check.py` builds scratch copies in a temp directory (never stamping tracked files) and runs each port against its original C++ submission (wrapping IOI grader submissions with thin C++ stdio drivers):

```text
python3 proto/ports/blind_critic_4/check.py              # all eight, 200 inputs each
python3 proto/ports/blind_critic_4/check.py ioi15p6 --iters 300
```
