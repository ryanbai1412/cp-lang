// expect: ok, output "1 3 5 2 4"
// Acceptance for seq/U45 (every k-th element), handbook/product/std/sequences-usecases.md: `a.step(k)` on a list, not the grid `step`.
let a = [1, 2, 3, 4, 5]
let odd = List(a.drop(1))
print(a.step(2).collect(), odd.step(2).collect())
