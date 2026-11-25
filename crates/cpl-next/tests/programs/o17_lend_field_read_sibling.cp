// expect: ok, output "4"
// (spec) R05.8.1
// False-rejection probe: lend one field `mut`, pass a sibling field by value.
struct P:
    a: List<Int>
    b: List<Int>
fn app(mut to: List<Int>, from: List<Int>):
    for x in from: to.push(x)
fn go() -> Int:
    var p = P([1, 2, 3], [4])
    app(mut p.b, p.a)
    p.b.len
print(go())
