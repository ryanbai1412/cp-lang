// expect: ok, output "5"
// Acceptance for seq/U8 (prefix folds over a slice), handbook/product/std/sequences-usecases.md.
print([3, 1, 4, 1, 5][1..].scan(+).fold(1..3))
