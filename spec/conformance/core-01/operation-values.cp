// expect: ok, output "5 5 6"
// rule: 01.5.5
fn twice(op: (Int, Int) -> Int, a: Int, b: Int) -> Int = op(op(a, b), b)
print(twice(+, 1, 2), twice(max, 1, 5), twice(gcd, 12, 18))
