// Source: Codeforces 2096D (Wonderful Lightbulbs), https://codeforces.com/contest/2096/problem/D
// Problem: from one lit bulb (the treasure), each operation toggles (x, y), (x, y + 1),
//   (x + 1, y - 1) and (x + 1, y). Given the n <= 2e5 lit bulbs (|coords| <= 1e8, multitest),
//   print any possible treasure position.
// Solution: an operation toggles two bulbs in each of columns x, x + 1 and two on each of the
//   diagonals x + y, x + y + 1, so the XOR of x and the XOR of x + y over lit bulbs never
//   change: they give the treasure's x and x + y. O(n).

@!multitest
let n: 1..=200_000 = read()
let pts: [0..n](x: -10 ** 9..=10 ** 9, y: -10 ** 9..=10 ** 9) = read()
let x = fold(^, p.x for p in pts)
let diag = fold(^, p.x + p.y for p in pts)
let y = diag - x
print(x, y)
