// expect: ok, output "3 12 3"
// rule: 04.2.30
// R04.2.30: `V.all` is every position of `V` in walk order, as a `Range<V>`, a member of the
// domain type.
domain V = 0..3
var s = 0
for v in V.all: s = s * 10 + v.ord
let r: Range<V> = V.all
print(V.all.len, s, r.len)
