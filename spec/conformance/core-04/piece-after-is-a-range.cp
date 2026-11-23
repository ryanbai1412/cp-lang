// expect: ok, output "3"
// rule: 04.2.19
// `v.after` is a `Range<D>` and goes where one is taken.
domain C = 0..5
var a: [C]Int = 1
fn total(a: [C]Int, r: Range<C>) -> Int = a[r].sum()
print(total(a, C(1).after))
