// Codeforces 2255 C (two-run communication): the first run hides a cell
// (r, c) by toggling the grid so that, for the minority colour with w cells,
// the row sum is r*w mod n and the column sum is c*w mod n; moving one
// minority cell by (dr, dc) shifts both sums.  The second run decodes it.
let op: Str = read()
@!multitest
let n: 1..=500 = read()
let rows: [0..n]Str = read()
let black = sum(count(ch == '#' for ch in row) for row in rows)
// work with the minority colour
let ink = if black > n * n - black: '.' else: '#'
let w = min(black, n * n - black)
let sr = sum(i for i in 0..n for j in 0..n if rows[i][j] == ink) % n
let sc = sum(j for i in 0..n for j in 0..n if rows[i][j] == ink) % n
if op == "second":
    print(
        first(r for r in 0..n if r * w % n == sr)! + 1,
        first(c for c in 0..n if c * w % n == sc)! + 1,
    )
    return
let (r, c): (1..=n, 1..=n) = read()
let dr = ((r - 1) * w - sr) % n
let dc = ((c - 1) * w - sc) % n
if dr == 0 and dc == 0:
    print(1, 1, 1, 1)
    return
for i in 0..n:
    for j in 0..n:
        let (ti, tj) = ((i + dr) % n, (j + dc) % n)
        if rows[i][j] == ink and rows[ti][tj] != ink:
            print(i + 1, j + 1, ti + 1, tj + 1)
            return
