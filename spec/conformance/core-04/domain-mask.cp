// expect: ok, output "NO YES NO YES"
// rule: 04.2.1
// D.mask(vs) builds a [D]Bool marking the positions yielded by vs.
domain V = 1..=4
let chosen: List<V> = [V(2), V(4)]
let m = V.mask(chosen)
print(yes(m[V(1)]), yes(m[V(2)]), yes(m[V(3)]), yes(m[V(4)]))
