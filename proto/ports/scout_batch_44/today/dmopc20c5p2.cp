// Runnable port (proto/cp.py lib-run) of dmopc20c5p2.cp
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
for cell in sorted(cells):
    print(cell.0 + 1, cell.1 + 1)
