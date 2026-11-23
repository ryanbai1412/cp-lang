// A scan over a domain-keyed view keeps the source's domain and walks only the view's keys.
domain V = 1..=5
let a = [V] v => Int(v) * 10
let p = a[V(2)..=V(4)].scan(+)
print(p.prefix(V.cuts(2)), p.prefix(V.cuts(3)), p.prefix(V.cuts(4)))
let q = a.scan(+)
print(q.prefix(V.cuts(3)), q.all)
