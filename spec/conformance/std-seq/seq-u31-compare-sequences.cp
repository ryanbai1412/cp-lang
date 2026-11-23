// expect: ok, output "YES YES 1"
// Acceptance for seq/U31 (compare two sequences), handbook/product/std/sequences-usecases.md.
let a = [1, 2, 3]
let b = [1, 5, 3]
print(yes(a < b), yes(a == a), a.keys.find(i => a[i] != b[i])!)
