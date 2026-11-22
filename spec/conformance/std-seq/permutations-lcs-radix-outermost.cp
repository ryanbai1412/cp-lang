// expect: ok, output "YES 6 1\nYES YES\nYES\n-70000 5 5 256 1000 99999\nYES\nYES YES"
use std.seq.{lcs}
print([1, 2].permutations().collect() == [[1, 2], [2, 1]], [1, 2, 3].permutations(k: 2).collect().len, [1, 2, 3].permutations(k: 0).collect().len)
print(lcs("abc", "acb").len == 2, lcs([1, 2, 3, 4], [2, 4]) == [(1, 0), (3, 1)])
var a = [3, -1, 2]
a.radix_sort()
print(a == [-1, 2, 3])
var b = [1000, 5, -70000, 256, 5, 99999]
b.radix_sort()
print(b)
var c = [(2, 1), (1, 2), (2, 0), (1, 1)]
c.radix_sort(by: p => p.0)
print(c == [(1, 2), (1, 1), (2, 1), (2, 0)])
print([0..5, 1..3, 4..8].outermost() == [0..5, 4..8], [1..3, 1..3, 0..2].outermost() == [0..2, 1..3])
