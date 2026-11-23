// expect: ok, output "1 3 0\n3"
// Acceptance for seq/U3 (position of the extreme, with a tie rule), handbook/product/std/sequences-usecases.md.
let a = [3, 5, 4, 5]
print(a.argmax()!, a.argmax(ties: Ties.Last)!, a.argmin()!)
print(a.keys.max(by: i => a[i], ties: Ties.Last)!)
