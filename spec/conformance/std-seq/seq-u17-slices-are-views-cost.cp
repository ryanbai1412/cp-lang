// expect: ok, output "1999998 3"
// Acceptance for seq/U17 (slices are views, at n = 10^6), handbook/product/std/sequences-usecases.md.
let n = 1000000
let t = "ab".repeat(n / 2)
var k = 0
for i in 0..n - 1:
    k += t[i..i + 2].len
let a = List.filled(n, 1)
var b: List<Int> = []
b.extend(a[n - 3..])
print(k, b.len)
