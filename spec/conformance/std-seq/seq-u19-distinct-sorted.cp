// expect: ok, output "3 1"
// Acceptance for seq/U19 (distinct values of sorted input), handbook/product/std/sequences-usecases.md.
print([3, 3, 1, 1].sorted(desc: true).distinct().collect())
