// expect: ok, output "5"
// rule: 04a.3.4 04b.1.2
domain V = 1..=5
let a = [V] v => Int(v)
let p = a[V(2)..=V(4)].scan(+)
print(p.prefix(V.cuts(3)))
