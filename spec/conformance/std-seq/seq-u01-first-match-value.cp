// expect: ok, output "4 1\n-1"
// Acceptance for seq/U1 (value of the first or last match), handbook/product/std/sequences-usecases.md.
let l = [3, 1, 4, 1, 5]
print(l.find(x => x > 3)!, l.rfind(x => x < 3)!)
print(l.find(x => x > 9) ?? -1)
