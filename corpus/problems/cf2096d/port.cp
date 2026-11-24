// Source: Codeforces 2096D (Wonderful Lightbulbs), https://codeforces.com/contest/2096/problem/D
// limits: 2000 ms, 250 MB
// Problem: from one lit bulb (the treasure), each operation toggles (x, y), (x, y + 1),
//   (x + 1, y - 1) and (x + 1, y). Given the n <= 2e5 lit bulbs (|coords| <= 1e8, multitest),
//   print any possible treasure position.
// Solution: an operation toggles two bulbs in each of columns x, x + 1 and two on each of the
//   diagonals x + y, x + y + 1, so the XOR of x and the XOR of x + y over lit bulbs never
//   change: they give the treasure's x and x + y. Fold them as the bulbs are read. O(n).
// Bounds: n: 1..=200_000; px, py: -1e9..=1e9

@!multitest
let n: 1..=200_000 = read()
var (x, diag) = (0, 0)
repeat n:
    let (px, py): (-1e9..=1e9, -1e9..=1e9) = read()
    x ^= px
    diag ^= px + py
print(x, diag - x)
