// expect: ok, output "1 2 5 4 2"
// Acceptance for seq/U34 (k-th smallest and the k smallest), handbook/product/std/sequences-usecases.md: `select` works in place, so it needs a `var`.
var a = [5, 1, 4, 2]
a.select(1)
print([5, 1, 4, 2].min_k(2), [5, 1, 4, 2].max_k(2), a[1])
