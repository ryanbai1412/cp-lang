// Codeforces 2152 B — Catching the Krug
//
// On `[0..=n] x [0..=n]`, the evader `(rk, ck)` moves orthogonally (L1 <= 1)
// while the pursuer `(rd, cd)` moves in Chebyshev metric (Linf <= 1). Because
// Chebyshev distance decouples across axes, the evader maximizes survival time
// by fleeing to the far boundary on each axis where it leads the pursuer:
// - If `rk < rd`, flee to `0` (pursuer needs `rd` steps to reach row `0`).
// - If `rk > rd`, flee to `n` (pursuer needs `n - rd` steps to reach row `n`).
// - If `rk == rd`, the pursuer matches every step on that axis (`dx = 0`).
// The column axis is symmetric, and capture takes `max(dx, dy)` turns.

@!multitest
let n: 1..=10 ** 9 = read()
let (rk, ck, rd, cd): (0..=10 ** 9, 0..=10 ** 9, 0..=10 ** 9, 0..=10 ** 9) = read()
let dx = if rd > rk: rd elif rd < rk: n - rd else: 0
let dy = if cd > ck: cd elif cd < ck: n - cd else: 0
print(max(dx, dy))
