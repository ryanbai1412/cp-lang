// expect: E0302
// rule: 06.2.5
fn twice(f: (Int) -> Int, x: Int) -> Int = f(f(x))
fn add(a: Int, b: Int) -> Int = a + b
print(twice(add, 3)) // <- here
