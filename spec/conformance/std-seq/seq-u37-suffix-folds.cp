// expect: ok, output "6"
// Acceptance for seq/U37 (suffix folds), handbook/product/std/sequences-usecases.md.
print([1, 2, 3].rev.scan(+).all)
