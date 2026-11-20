// DMOPC '20 Contest 5 P2 — cells cut by the diagonal of an n x m grid.
// original: corpus/submissions/3591332.cpp
let (n, m): (Int, Int) = read()

var cells = Set<(Int, Int)>()
if n == m: cells.extend((i, i) for i in 0..n)
// The diagonal crosses column line c strictly inside row n*c/m, and vice versa.
for c in 0..=m if n * c % m != 0:
    cells.extend([(n * c / m, c - 1), (n * c / m, c)])
for r in 0..=n if m * r % n != 0:
    cells.extend([(r - 1, m * r / n), (r, m * r / n)])

print(cells.len)
for (r, c) in cells: print(r + 1, c + 1)
