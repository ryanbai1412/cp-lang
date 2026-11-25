// expect: ok, output "3"
// rule: R06 defaults
// A default parameter value.
fn add(a: Int, b: Int = 2) -> Int = a + b
print(add(1))
