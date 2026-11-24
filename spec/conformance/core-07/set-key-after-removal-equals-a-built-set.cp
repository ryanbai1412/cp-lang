// expect: ok, output "2 7 2"
// rule: 07.12.9
// A `Set` key that reached its members through a removal equals one built with them, and the empty set is its own key (adversarial round 47).
domain Q = 0..5
var a: Set<Q> = Set([Q(1), Q(2), Q(3)])
a.remove(Q(3))
let e: Set<Q> = Set()
var m: Map<Set<Q>, Int> = Map()
m[a] = 1
m[Set([Q(2), Q(1)])] = 7
m[e] = 2
print(m.len, m.get(a) ?? 0, m.get(Set()) ?? 0)
