// expect: ok, output "1 2 1 0 -1"
// Acceptance for seq/U6 (bounds in sorted data), handbook/product/std/sequences-usecases.md.
let a = [1, 3, 5]
print(a.first_ge(3)!, a.first_gt(3)!, a.last_le(4)!, a.last_lt(3)!, a.first_ge(6) ?? -1)
