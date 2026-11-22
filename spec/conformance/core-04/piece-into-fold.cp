// expect: ok, output "5\n9"
// rule: 04.4.21
domain V = 1..=5
let a = [V] v => Int(v)
let pre = a.scan(+)
print(pre.sum(V.cuts(1)..V.cuts(3)))
print(a[V(2)..=V(4)].sum())
