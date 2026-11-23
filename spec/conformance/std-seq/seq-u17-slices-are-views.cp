// expect: ok, output "1 2 4 5"
// Acceptance for seq/U17 (slices are views), handbook/product/std/sequences-usecases.md.
var g = [1, 2]
g.extend([3, 4, 5][1..])
print(g)
