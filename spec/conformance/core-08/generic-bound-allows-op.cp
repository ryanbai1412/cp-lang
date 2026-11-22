// expect: ok, output "6"
// rule: 08.2.2
fn twice<T: Add>(x: T) -> T = x + x
print(twice(3))
