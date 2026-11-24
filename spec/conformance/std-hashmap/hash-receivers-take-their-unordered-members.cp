// expect: ok, output "2 YES 20 | 2 3 9 NO"
// A `HashMap`/`HashSet` takes every member that needs no order: lookups, `len`, `remove`,
// `min`/`max` (a scan) and `sorted`.
var h = HashMap<Int, Int>()
h[1] = 10
h[2] = 20
h.remove(1)
h[3] = 30
var s = HashSet<Int>()
s.insert(9)
s.insert(3)
print(h.len, yes(h.contains(3)), h.get(2) ?? 0, "|", s.len, s.min()!, s.max()!, yes(s.contains(4)))
