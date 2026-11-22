// Codeforces 2255 A — Circular Adjacent Pairs
//
// On a circle of `m = 2 * n` cells, a cell `i` counts when it is `'1'`
// followed by `'1'` or `'0'` preceded by `'1'`. The input parameter `_k` is
// read and unused because the parity counts of valid cells are invariant under
// the shift parameter `k`. `(i - 1) % m` uses floor modulo, so `(0 - 1) % m == m - 1`
// wraps cleanly around the circle.

@!multitest
let (n, _k): (1..=200_000, 1..=10 ** 9) = read()
let m = 2 * n
let s: [0..m]Char = read()

fn hit(i: Int) -> Bool:
    (s[i] == '1' and s[(i + 1) % m] == '1') or (s[i] == '0' and s[(i - 1) % m] == '1')

print(count(hit(i) for i in 1..m by 2), count(hit(i) for i in 0..m by 2))
