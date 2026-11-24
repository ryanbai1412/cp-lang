# ioi15p5: Sorting

## No permutation type: a row and its inverse are swapped in sync by hand
- **Where:** `port.cp`, `struct Row(val: [Pos]Pos, at: [Pos]Pos)` and `fn swap`, which swaps
  `val` and patches `at` after every move.
- **What felt wrong:** a permutation with "where is value v" is a standard structure; keeping the
  inverse by hand is two facts written in two places, and a forgotten patch is a wrong answer.
- **Goal it hurts:** hard-to-get-wrong, express.
- **Ideal code (`dream.cp`):** `let start = Perm<Pos>(start)`, `now.swap(x, y)`, `now.where(u)`.
- **General fix:** a std `Perm<D>` holding both directions, with `swap`, `where`, `inverse`,
  `cycles` and `sorting_swaps`; helps every swap-simulation port (ioi15p5, cf permutation tasks).
- **Severity:** medium.

## Folding a list of swaps over a value
- **Where:** `fn after(r)`, a clone plus a loop of `swap_at`.
- **What felt wrong:** the dream's `ermek.take(r).fold(start, (mut p, e) => p.swap(e.x, e.y))`
  says "apply these swaps"; the loop is fine but longer.
- **Goal it hurts:** express.
- **Ideal code:** `ermek.take(r).fold(start.clone(), (mut p, e) => p.swap(e.x, e.y))`.
- **General fix:** `fold` with a `mut` accumulator (in-place step), so a fold over moves costs no copies.
- **Severity:** low.


## Measurements

Contestant-written tokens: port 343, original 584 (`corpus/audit.py` count). Emitted C++: 22413 bytes (`cpl build`, judge `generic`).
Max input not timed: the accepted C++ is a grader function (`findSwapPairs`) with no main.
