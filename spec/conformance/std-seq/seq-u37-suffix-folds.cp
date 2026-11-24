// expect: ok, output "6"
// gap: stdlib 2026-11-24T05Z
// Acceptance for seq/U37 (suffix folds), handbook/product/std/sequences-usecases.md.
print([1, 2, 3].rev.scan(+).all)
