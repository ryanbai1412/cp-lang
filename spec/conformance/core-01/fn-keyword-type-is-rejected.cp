// expect: E0503
// rule: 01.10.2
// has: "->"
// A fn type is `V -> V`, with no `fn`.
fn twice(f: fn(Int) -> Int, x: Int) -> Int = f(f(x)) // <- here
print(twice(x => x + 1, 3))
