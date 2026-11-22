// expect: ok, output "3 14 15"
// rule: 04.4.26 04.8.8
domain V = 1..=5
fn total(a: [V]Int, r: Range<V>) -> Int = a[r].sum()
let a: [V]Int = [1, 2, 3, 4, 5]
print(total(a, 1..3), total(a, 2..=5), total(a, 1..6))
