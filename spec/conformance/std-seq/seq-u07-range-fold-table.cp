// expect: ok, output "5 4 14"
// Acceptance for seq/U7 (range folds from one prefix table), handbook/product/std/sequences-usecases.md.
let p = [3, 1, 4, 1, 5].scan(+)
print(p.fold(1..3), p.prefix(2), p.all)
