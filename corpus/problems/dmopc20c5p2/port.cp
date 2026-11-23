// Source: DMOPC '20 Contest 5 P2 (On The Clock), https://dmoj.ca/problem/dmopc20c5p2
// Problem: an N x M pixel grid (N, M <= 1e6), and the segment from its bottom-left to its
//   top-right corner. Print the number of pixels whose interior it crosses (touching a
//   corner does not count), then those pixels sorted by (row, column).
// Solution: the line meets the gridline x = c at height n c / m; if that is not an integer,
//   it crosses into the pixels on both sides in row floor(n c / m). Same for the
//   horizontal gridlines; only when n = m does it go corner to corner, through the
//   diagonal. Collect in a set and sort: O((n + m) log(n + m)).
// original: corpus/submissions/3591332.cpp

let (n, m): (Int, Int) = read()

var cells = Set<(Int, Int)>()
if n == m:
    for i in 0..n: cells.add((i, i))
for c in 0..=m:
    if n * c % m != 0:
        let r = n * c / m
        cells.add((r, c - 1))
        cells.add((r, c))
for r in 0..=n:
    if m * r % n != 0:
        let c = m * r / n
        cells.add((r - 1, c))
        cells.add((r, c))

print(cells.len)
for cell in cells.sorted():
    print(cell.0 + 1, cell.1 + 1)
