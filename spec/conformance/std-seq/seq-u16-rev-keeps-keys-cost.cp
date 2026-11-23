// expect: ok, output "499999500000"
// Acceptance for seq/U16 (walk backwards and keep the keys, at n = 10^6), handbook/product/std/sequences-usecases.md.
let n = 1000000
let a = List.filled(n, 1)
var s = 0
for (k, x) in a.rev.items:
    s += k * x
print(s)
