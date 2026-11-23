// expect: ok, output "0 0 0 0 2 4 0 2 4"
// Acceptance for seq/U65 (build an array or list), handbook/product/std/sequences-usecases.md.
domain I = 0..3
let a: [I]Int = [I] i => Int(i) * 2
print(List.filled(3, 0), a, List((0..3).map(x => x * 2)))
