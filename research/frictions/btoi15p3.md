# btoi15p3 friction

Rethought from dream.cp. The v0.2 port picked the root with a `var` loop and spelled `Graph<V, E>.undirected` and `Rooted(u => g.out(u), ..)`. The v0.3 port is the dream: `V.find(v => g.deg(v) > 1)`, `Rooted(g, ..)`, and `half` named once. `g.deg` on `Graph.undirected` now works, so that earlier friction is gone.

## The corpus suite cannot accept a second optimal answer

- **Where:** `cpl-conform corpus btoi15p3`. Any set of ceil(L / 2) links that leaves no
  bridge is accepted; `spec.cp` links leaf i to leaf i + ceil(L / 2) in preorder, the AC
  C++ pairs leaves greedily in its DFS. `crates/cpl-conform/src/corpus.rs` compares tokens
  with `tests/NN.ans` and never runs the problem's `checker.py`, so 4 of 8 cases read
  `wrong` though `checker.py` accepts all 8 (and the AC answers).
- **Repro:** `printf '5\n1 2\n1 3\n1 4\n1 5\n'` has optimal answers `2 / 2 3 / 4 5` and
  `2 / 2 4 / 3 5`; the suite accepts only the reference's.
- **Goal it hurts:** dependable evidence (a correct, idiomatic solution reads as wrong; the
  only way to pass is to copy the reference's tie-breaking).
- **General fix:** when `corpus/problems/<id>/checker.py` exists, the corpus runner calls
  `checker.py input output answer` and passes on exit 0, as the port checks already do.
  Routed to spec (owner of `crates/cpl-conform`).
- **Status:** open.

## Measurements

gen.py --max: port 0.26 s. The accepted C++ recurses and dies (signal 11) under an 8 MB stack; with an unlimited stack it takes 0.27 s. checker.py accepts the port's links.
