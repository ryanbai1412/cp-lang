// expect: ok, output "1 2 3 30 1 22"
// Acceptance for seq/U51 (linear-time sorts), handbook/product/std/sequences-usecases.md: both sort in place, so they need a `var`; `counting_sort` keys by a domain and is stable.
domain Dig = 0..10
var a = [3, 1, 2]
a.radix_sort()
var b = [30, 1, 22]
b.counting_sort(by: x => Dig(x % 10))
print(a, b)
