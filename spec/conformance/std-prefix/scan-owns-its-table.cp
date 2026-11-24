// expect: ok, output "6 5 100 2 0"
// rule: 07.4.1
// A scan owns its table: writing the source afterwards, whole or through a slice it was taken from, leaves the sums unchanged.
var a = [1, 2, 3]
let p = a.scan(+)
a[0] = 100
let q = a[1..].scan(+)
a[2] = 0
print(p.prefix(3), q.prefix(3), a)
