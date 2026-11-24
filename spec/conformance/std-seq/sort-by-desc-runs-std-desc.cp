// expect: ok, output "5 3 3 1\n2 1 0 3\ncc b a\n1 0 2"
// rule: 07.9.3
// `sort(by: x => desc(k))` orders by std's `Desc` Ord impl, largest key first, and keeps equal
// keys in input order; on ints, on records with a tiebreak id, and on strings.
use std.algebra.{desc}
var a = [3, 1, 5, 3]
a.sort(by: x => desc(x))
print(a[0], a[1], a[2], a[3])
struct P(v: Int, id: Int)
var ps = [P(1, 0), P(2, 1), P(3, 2), P(1, 3)]
ps.sort(by: p => desc(p.v))
print(ps[0].id, ps[1].id, ps[2].id, ps[3].id)
var ss = ["a", "cc", "b"]
ss.sort(by: s => desc(s))
print(ss[0], ss[1], ss[2])
var ks = [(1, 2), (1, 3), (0, 5)]
var ids = [0, 1, 2]
ids.sort(by: i => desc(ks[i]))
print(ids[0], ids[1], ids[2])
