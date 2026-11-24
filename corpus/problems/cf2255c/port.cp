// Source: Codeforces 2255C (Even If the World Turns),
//   https://codeforces.com/contest/2255/problem/C
// Problem: run twice, multitest, n <= 800 (sum of n^2 <= 800^2). First run: an n x n '#'/'.'
//   picture with w black cells, gcd(n, w) = 1, and a target cell; swap the colours of two cells.
//   The jury then shifts cyclically, rotates, reflects and inverts the picture (the target moves
//   with it). Second run: given only the final picture, print where the target is.
// Solution: let ink be the minority colour (count w; inversion keeps it, as n^2 = 2w is ruled
//   out by the gcd) and (sr, sc) its coordinate sums mod n. Since gcd(n, w) = 1 the "centroid"
//   (sr/w, sc/w) mod n is a cell, and every allowed move maps it like a cell. First run: move one
//   ink cell by (dr, dc) onto a non-ink cell so the centroid lands on the target; such a cell
//   exists because ink shifted by (dr, dc) != 0 can't equal itself when gcd(n, w) = 1. Second
//   run: solve r * w = sr, c * w = sc (mod n). O(n^2) per run.
// Bounds: n: 2..=800; r, c: 1..=n

let op: Str = read()

@!multitest
let n: 2..=800 = read()
let g: [0..n]Str = read()
let black = sum(row.count(c => c == '#') for row in g)
let ink = if black > n * n - black: '.' else: '#'
let w = min(black, n * n - black)
let sr = sum(i for i in 0..n for j in 0..n if g[i][j] == ink) % n
let sc = sum(j for i in 0..n for j in 0..n if g[i][j] == ink) % n

if op == "second":
    print((0..n).find(r => r * w % n == sr)! + 1, (0..n).find(c => c * w % n == sc)! + 1)
    return

let (r, c): (1..=n, 1..=n) = read()
let (dr, dc) = (((r - 1) * w - sr) % n, ((c - 1) * w - sc) % n)
if dr == 0 and dc == 0:
    print(1, 1, 1, 1)
    return
let (si, sj) = first(
    (i, j) for i in 0..n for j in 0..n if g[i][j] == ink and g[(i + dr) % n][(j + dc) % n] != ink
)!
print(si + 1, sj + 1, (si + dr) % n + 1, (sj + dc) % n + 1)
