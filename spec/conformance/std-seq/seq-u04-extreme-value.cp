// expect: ok, output "-1 3 1 6"
// Acceptance for seq/U4 (extreme value, possibly empty), handbook/product/std/sequences-usecases.md.
let e: List<Int> = []
print(e.max() ?? -1, [3, 1].max()!, [3, 1].max(by: x => -x)!, max(i * (5 - i) for i in 0..6)!)
