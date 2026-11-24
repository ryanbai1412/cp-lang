// expect: ok, output "YES YES\nYES YES\nYES YES"
// rule: 07.4.14
// `s.cut_after(i)` and `s.cut_before(i)` are the cuts on either side of key `i` in walk order: `i.r` and `i.l` forward, swapped on `.rev`, and a slice keeps its parent's cuts.
domain V = 1..=4
let a = [V] v => Int(v)
let v = V(2)
print(yes(a.cut_after(v) == v.r), yes(a.cut_before(v) == v.l))
print(yes(a.rev.cut_after(v) == v.l), yes(a.rev.cut_before(v) == v.r))
let w = a[V(2)..=V(3)]
print(yes(w.cut_after(V(3)) == V(3).r), yes(w.rev.cut_after(V(2)) == V(2).l))
