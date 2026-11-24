// expect: ok, output "1\n2\n3\n4\n5"
// rule: 01.6.7
// `todo` never gives a value, so it fits any place one goes.
fn f() -> Int = todo
fn g(c: Bool) -> Int = if c: 2 else: todo
fn h(x: ?Int) -> Int = x ?? todo
fn k(n: Int) -> Int = match n:
    4 => 4
    _ => todo
fn m(c: Bool) -> Int:
    let x: Int = if c: 5 else: todo
    x
fn n() -> Int:
    let y: Int = todo
    y + 1
let call = false
if call: print(f() + n())
print(1)
print(g(true))
print(h(Some(3)))
print(k(4))
print(m(true))
