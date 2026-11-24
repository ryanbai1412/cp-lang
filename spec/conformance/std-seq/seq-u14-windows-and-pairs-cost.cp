// expect: ok, output "2999994"
// Acceptance for seq/U14 (fixed windows, chunks and adjacent pairs, at n = 10^6), handbook/product/std/sequences-usecases.md.
let n = 1000000
let a = List.filled(n, 1)
var s = 0
for w in a.windows(3):
    s += w.sum()
for (x, y) in a.adjacent:
    s += y - x
print(s)
