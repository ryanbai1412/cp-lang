// expect: ok, output "3"
// (spec) R05.8.1
// False-rejection probe: two `mut` lends of disjoint fields of one struct in one call.
struct P:
    a: List<Int>
    b: List<Int>
fn mv(mut from: List<Int>, mut to: List<Int>):
    for x in from: to.push(x)
    from = []
fn go() -> Int:
    var p = P([1, 2, 3], [])
    mv(mut p.a, mut p.b)
    p.a.len * 10 + p.b.len
print(go())
