// expect: ok, output "10 9 20"
// rule: 07.5.11
domain V = 1..=4
fn total(xs: [V]Int) -> Int = xs.sum()
let a = [V] v => Int(v)
print(total(a), total(a[V(2)..]), total(a.map(x => x * 2)))
