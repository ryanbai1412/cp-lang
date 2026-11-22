// expect: E0302 at 3
// rule: 04.4.6
domain V = 1..=5
let a = [V] v => Int(v)
print(a[V.cuts(1)..=V.cuts(3)].sum())
