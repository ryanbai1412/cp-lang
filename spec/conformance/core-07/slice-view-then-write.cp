// expect: ok, output "2 5\n9"
// rule: 07.2.6
// A slice is a view: made in O(1), it reads the source; once it is dead the source may change.
var xs = [5, 1, 4, 2]
let v = xs[1..3]
print(v.len, v.sum())
xs[1] = 9
print(xs[1])
