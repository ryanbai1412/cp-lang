// expect: E0602
// rule: 07.1.5 05.7.5
// `next_permutation` takes `mut self`, so on a view it changes the source while `v` still reads it.
var a = [3, 1, 2]
let v = a[0..2]
a[0..2].next_permutation() // <- here
print(v.sum())
