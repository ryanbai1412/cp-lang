# Blind Critic #5 — CP Lang v2.2 Verification Ports

Six hard, previously unported Olympiad problems written blindly against the locked CP Lang v2.2 one-page cheat sheet (`docs/one-page-cheat-sheet.md` at `16c07bf`, 34 keywords), paired with runnable forms verified through the Rust `cpl` compiler (`c521b2c`).

## Layout

- `*_draft.cp` — Frozen blind first drafts written strictly from `docs/one-page-cheat-sheet.md` before reading `crates/`, `proto/cpc/`, `stdlib/`, `decisions.md`, or `research/`.
- `<problem>.cp` — Runnable CP Lang v2.2 ports that compile cleanly under the real Rust `cpl check`, `cpl run`, and `cpl build` (`g++ -std=c++17 -O2 -fsanitize=address,undefined`).
- `check.py` — Triple differential test harness comparing each original accepted C++ submission in `corpus/submissions/` against both `cpl build` (ASan/UBSan) and `cpl run` on randomized test cases.

## Problems

- `bsspc21s4` (`4152932.cpp`) — 2D rectangle sweep with non-pushdown cover-count segment tree and trailing-zero interval merging.
- `noi22p5` (`4832632.cpp`) — Coordinate compression, active multiset sweep, greedy interval stabbing, order-statistic inversion counting, and lazy range-add/min segment tree.
- `noi04p3` (`5032868.cpp`) — Horizontal bitmask enumeration with backward column DP and path reconstruction using tropical `?Int`.
- `noi08p4` (`5144322.cpp`) — Functional-graph cycle enumeration with floating-point max-plus tree knapsack DP.
- `noiwc22p3` (`4261177.cpp`) — Stateful Wordle grader with candidate filtering and minimax bucket scoring.
- `ioi19p5` (`5664246.cpp`) — Diagonal-projection Boolean circuit construction (`add_and`, `add_or`, `add_not`).

## Running Verification

Build the Rust CLI once, then run the differential suite:

```bash
cargo build -p cpl-cli
python3 proto/ports/blind_critic_5/check.py --iters 30 --interp-iters 5
```
