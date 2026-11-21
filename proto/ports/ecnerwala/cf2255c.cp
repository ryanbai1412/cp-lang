// Codeforces 2255 C (two-run communication): the first run hides a cell
// (r, c) by toggling the grid so that, for the minority colour with w cells,
// the row sum is r*w mod n and the column sum is c*w mod n; moving one
// minority cell by (dr, dc) shifts both sums.  The second run decodes it.
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
    print(
        first(r for r in 0..n if r * w % n == sr)! + 1,
        first(c for c in 0..n if c * w % n == sc)! + 1,
    )
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
