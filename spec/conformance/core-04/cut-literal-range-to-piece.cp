// expect: ok, output "5"
// rule: 04.4.26
domain V = 0..5
fn total(a: [V]Int, r: Range<V>) -> Int = a[r].sum()
let a: [V]Int = [1, 2, 3, 4, 5]
print(total(a, 1..3))
