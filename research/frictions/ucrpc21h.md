# ucrpc21h friction

## A two-parameter closure into `flat_map` loses the inner element type

- **Where:** `corpus/problems/ucrpc21h/spec.cp` wants the answer as
  `can.items.flat_map(((a, b), cs) => cs.each().map(c => ...)).min()`; it walks
  `(Load, Load).flat_map(((a, b)) => can[a, b].each().map(...))` instead.
- **Repro:** `let ps = [(1, [1, 2]), (2, [3])]; print(ps.flat_map((a, cs) => cs.map(c => a + c)).sum())`
  is E0302 "expected `Int`, found `List<Int>`"; the one-parameter `p => p.1.map(c => p.0 + c)`
  prints 10, and `.map((a, cs) => ...)` with the same closure checks.
- **Goal it hurt:** express (a natural destructuring closure needs a different walk).
- **Routed:** compiler worklog.

## The dense 3-D `[Split]Bool` subset-sum was too slow to judge

- The old spec walked every `(a, b, c)` cell per distance: case 08 (total 189) took 60 s in
  `cpl run`, past the 20 s corpus limit. Keeping the third load as a `Bitset<Load>` per `(a, b)`
  cell (`shifted`, `union`) does the same DP 64 cells per word: 0.9 s. Not a gap: the bitset is
  the right algorithm, and what the C++ submissions use.
