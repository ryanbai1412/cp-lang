// expect: E0602
// rule: 07.1.5 05.7.5
// An in-place verb on a view mutates its source like `mut a[i]` (R07.1.5), so it overlaps another live view of that source.
var a = [3, 1, 2]
let v = a[0..2]
a[0..2].sort() // <- here
print(v.sum())
