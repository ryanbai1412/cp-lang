// expect: ok, output "6"
// rule: 06.4.11
// R06.4.11: a documented decorator such as `@aka(..)` is accepted where it acts.
@aka(total)
fn sum3(a: Int, b: Int, c: Int) -> Int = a + b + c
print(sum3(1, 2, 3))
