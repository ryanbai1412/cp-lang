// expect: ok, output "1 1 0"
// Acceptance for seq/U3 (position of the extreme, with a tie rule), handbook/product/std/sequences-usecases.md.
let a = [3, 5, 4, 5]
print(a.keys.max(by: i => a[i])!, a.argmax()!, a.argmin()!)
