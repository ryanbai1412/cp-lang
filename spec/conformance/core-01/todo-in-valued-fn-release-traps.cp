// expect: exit 3
// rule: 01.6.7
// tool: run --release
// has: "`todo` reached"
fn f(x: Int) -> Int = todo
print(f(1))
