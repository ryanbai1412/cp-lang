// Codeforces 1985D Manhattan Circle
// in: t; per case n m, then n rows of m chars '.'/'#'. out: per case the center row and column (1-based)
@!multitest
let (n, m) = read<(1..=2e5, 1..=2e5)>()
domain R = 1..=n
domain C = 1..=m
let g = read<[R, C]Char>()
let rows = [Int(r) for r in R, c in C if g[r, c] == '#']
let cols = [Int(c) for r in R, c in C if g[r, c] == '#']
print((rows.min()! + rows.max()!) / 2, (cols.min()! + cols.max()!) / 2)
