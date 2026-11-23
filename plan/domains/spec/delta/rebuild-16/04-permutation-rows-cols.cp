// Codeforces 1980E Permutation of Rows and Columns: can row and column swaps turn a into b?
// In: t; per test `n m`, n rows of a, then n rows of b (each a permutation of 1..n*m). Out: YES/NO.
@!multitest
let n, m: 1..=2e5 = read()
domain R = 0..n
domain C = 0..m
domain Val = 1..=n * m
let a = read<[R, C]Val>()
let b = read<[R, C]Val>()

var spot: [Val]?(R, C) = None
for p in (R, C): spot[b[p]] = p
let at = spot!

// Swaps keep each row's set of values together, and each column's.
let rows = R.all(r => a[r].adjacent.all((x, y) => at[x].0 == at[y].0))
let cols = C.all(c => a[.., c].adjacent.all((x, y) => at[x].1 == at[y].1))
print(yes(rows and cols))
