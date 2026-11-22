// Codeforces 2255 C — Two-Run Torus Grid Encoding
//
// In an n x n grid of '#' and '.', let `ink` be the minority colour (count
// `w <= n^2 / 2`) and `(sr, sc)` be the sum of its row and column indices mod n.
// First run: hide `(r, c)` by shifting one `ink` cell by `(dr, dc)` mod n so
// the new sums satisfy `sr' == (r - 1) * w (mod n)` and `sc' == (c - 1) * w
// (mod n)` — whenever `(dr, dc) != (0, 0)`, a valid `ink` cell whose shifted
// target is non-`ink` exists because `ink` is a strict minority.
// Second run: recover `(r, c)` from `(sr, sc)` via `r * w == sr (mod n)` and
// `c * w == sc (mod n)`.

let op: Str = read()

@!multitest
let n: 1..=500 = read()
let g: [0..n]Str = read()
let black = sum(row.count('#') for row in g)
let ink = if black > n * n - black: '.' else: '#'
let w = min(black, n * n - black)
let sr = sum(i for i in 0..n for j in 0..n if g[i][j] == ink) % n
let sc = sum(j for i in 0..n for j in 0..n if g[i][j] == ink) % n

if op == "second":
    print((0..n).first(r => r * w % n == sr)! + 1, (0..n).first(c => c * w % n == sc)! + 1)
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
