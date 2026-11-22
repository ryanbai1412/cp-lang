// expect: ok, output "YES\nYES YES\n3\nYES YES\nYES\nYES YES"
use std.flow.{max_matching, min_vertex_cover, Matching}
domain L = 0..2
domain R = 0..2
let adj: [L]List<R> = [[R(0), R(1)], [R(0)]]
let mm = max_matching(adj)
print(yes(mm.size == 2 and mm.mate[L(0)] == Some(R(1)) and mm.mate[L(1)] == Some(R(0)) and mm.back[R(0)] == Some(L(1))))
let c = min_vertex_cover(adj)
print(yes(c.left[L(0)] and c.left[L(1)]), yes(not c.right[R(0)] and not c.right[R(1)]))
domain A = 0..4
domain B = 0..4
var m = Matching<A, B>.new()
m.add(A(0), B(0))
m.add(A(1), B(0))
m.add(A(1), B(1))
m.add(A(2), B(1))
m.add(A(2), B(2))
m.add(A(3), B(2))
print(m.augment_all())
print(yes(m.len == 3), yes(m.augment(A(3)) == false))
m.add(A(3), B(3))
let _ = m.augment(A(3))
let _ = m.augment_all()
print(yes(m.len == 4))
print(yes(m.remove(A(0)) == 4), yes(m.back(B(0)) == Some(A(0))))
