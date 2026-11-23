// expect: ok, output "999999 0 999999"
// Acceptance for seq/U12 (sort or sorted by key, descending, stable, at n = 10^6), handbook/product/std/sequences-usecases.md.
let n = 1000000
var a = [0..n] i => (i * 7919) % n
a.sort(by: x => -x)
let ord = a.keys.sorted(by: i => a[i])
print(a[0], a[n - 1], ord[0])
