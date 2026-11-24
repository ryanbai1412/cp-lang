// expect: ok, output "81 4"
// rule: 08.1.10
// A parameter typed `type_of(f)` takes the fn item `f` itself, and calls it.
fn sq(x: Int) -> Int = x * x
fn twice(f: type_of(sq), x: Int) -> Int = f(f(x))
fn half(x: Int) -> Int = x / 2
fn on(f: type_of(half), x: Int) -> Int = f(x)
print(twice(sq, 3), on(half, 9))
