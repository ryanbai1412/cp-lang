// expect: ok, output "6"
// rule: 08.2.4
fn total<domain I>(a: [I]Int) -> Int = a.sum()
domain V = 1..=3
let a = [V] v => Int(v)
print(total(a))
