// Codeforces 2147 A — Shortest Increasing Path
//
// Reach `(x, y)` with alternating `+x`, `+y`, `+x`, ... moves of strictly
// increasing positive lengths:
// - If `x < y`: 2 moves `(x, 0)` then `(0, y)` satisfy `0 < x < y` -> `2`.
// - If `x >= y`: 2 moves are impossible (`x < y` fails). For 3 moves
//   `(x1, 0), (0, y), (x - x1, 0)` we need `1 <= x1 < y < x - x1`, which is
//   satisfiable (e.g. `x1 = 1`) iff `y >= 2` and `x - y >= 2` -> `3`,
//   otherwise impossible (`-1`).

@!multitest
let (x, y): (1..=10 ** 9, 1..=10 ** 9) = read()
print(if x < y: 2 elif x - y <= 1 or y == 1: -1 else: 3)
