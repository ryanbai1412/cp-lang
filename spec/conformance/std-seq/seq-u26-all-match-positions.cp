// expect: ok, output "1 3"
// Acceptance for seq/U26 (every position that matches), handbook/product/std/sequences-usecases.md.
let a = [3, 0, 4, 0]
print(a.keys.filter(i => a[i] == 0).collect())
