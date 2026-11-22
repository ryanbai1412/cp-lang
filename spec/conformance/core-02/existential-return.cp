// expect: ok, output "3 4"
// rule: 02.9.3
fn doubled(n: 1..=10) -> (domain I, [I]Int):
    domain J = 0..n
    (J, [J] j => 2 * Int(j))
let (domain I, a) = doubled(3)
print(I.count, a[I(2)])
