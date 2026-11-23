// expect: ok, output "2 3"
// Acceptance for seq/U2 (position of the first or last match), handbook/product/std/sequences-usecases.md.
domain I = 0..5
let a: [I]Int = [3, 1, 4, 1, 5]
let i: I = a.keys.find(i => a[i] > 3)!
print(i, a.keys.rfind(i => a[i] == 1)!)
