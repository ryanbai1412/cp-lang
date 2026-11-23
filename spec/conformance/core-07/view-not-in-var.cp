// expect: ok, output "1\n2"
// rule: 07.5.10 05.7.8
// A view is a lent value: a `var` declared after its source in the same block may hold it, and be reassigned to another view of a live source.
let a = [1, 2]
var w = a[0..1]
print(w.sum())
w = a[1..2]
print(w.sum())
