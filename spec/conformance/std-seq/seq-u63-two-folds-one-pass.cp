// expect: ok, output "285 81"
// Acceptance for seq/U63 (two folds over one generator), handbook/product/std/sequences-usecases.md.
let g = (0..10).map(x => x * x)
print(g.sum(), g.max()!)
