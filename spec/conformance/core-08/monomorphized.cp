// expect: ok, output "3 a"
// rule: 08.2.3 13.3.1
fn same<T>(x: T) -> T = x
print(same(3), same("a"))
