let n: 1..=10 = read()
domain V = 1..=n
let a: [V]Int = read()
let v = V(2)
let r = V(2)..=V(4)
print(r.len, r.is_empty, sum(a[i] for i in r.rev), sum(a[i] for i in ..v), sum(a[i] for i in v..), sum(a[i] for i in ..=v))
