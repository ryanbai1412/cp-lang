// expect: ok, output "3 50"
// A small struct of scalars passed by value while the callee writes the global it came from.
struct Pt:
    x: Int
    y: Int
var p = Pt(1, 2)
fn f(q: Pt) -> Int:
    p = Pt(50, 60)
    q.x + q.y
print(f(p), p.x)
