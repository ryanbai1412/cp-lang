// expect: ok
// rule: 13.3.1 13.3.2
// tool: build -o -
// lacks: "template"
// lacks: "std::function"
fn twice<T>(x: T, f: T -> T) -> T = f(f(x))
print(twice(3, x => x * 2), twice("a", s => s + "b"))
