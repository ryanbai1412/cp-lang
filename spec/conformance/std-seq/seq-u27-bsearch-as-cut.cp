// expect: ok, output "2 2"
// Acceptance for seq/U27 (binary search answered as a position or a cut), handbook/product/std/sequences-usecases.md.
let a = [1, 2, 5, 7]
print(a.keys.first_true(i => a[i] >= 4)!, a.partition_point(x => x >= 4))
