// expect: ok, output "5 2 3 4 1"
// Acceptance for seq/U47 (sort part of an array), handbook/product/std/sequences-usecases.md.
var a = [5, 4, 3, 2, 1]
a[1..4].sort()
print(a)
