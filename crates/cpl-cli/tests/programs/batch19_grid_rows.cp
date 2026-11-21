// Syntax Batch #19: a row of a grid against a row, cell by cell.
domain K = 0..3
var g: [K, 0..4]Int = 0
for r in K:
    for c in 0..4: g[r, c] = r.idx * 3 + (c * 5) % 4
let (k, v): (K, K) = (K(0), K(2))
g[k, ..] max= g[v, ..]
print(g[k, ..])
if g[K(1), ..] min= g[k, ..]: print("changed")
print(g[K(1), ..])
print(g[k, ..] max= g[v, ..])
var h: [1..=3, 1..=4]Int = [[1, 9, 3, 7], [5, 2, 8, 0], [4, 4, 4, 4]]
h[.., 2] min= h[.., 4]
h[1, 2..=3] max= h[2, 3..=4]
h[3, ..] max= [9, 0, 9, 0]
h[2, ..] min= g[v, ..]
for r in 1..=3: print(h[r, ..])
var f: [K, K]F64 = 1.5
f[K(0), ..] max= f[K(1), ..]
let row: [K]F64 = [2.5, 0.5, 3.5]
print(f[K(2), ..] max= row, f[K(2), ..])
