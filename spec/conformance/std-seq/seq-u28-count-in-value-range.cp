// expect: ok, output "1..3"
// Acceptance for seq/U28 (how many sorted values fall in `[lo, hi)`), handbook/product/std/sequences-usecases.md.
let a = [1, 3, 3, 5, 8]
print(a.equal_range(3))
