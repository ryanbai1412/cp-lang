// expect: ok, output "1121"
// Two lenders active at once on two different structs: a.at(0) += b.at(0) is not expressible; use nesting through a fn.
struct C:
    v: Int
    fn slot(mut self) -> lends mut Int:
        lend mut v
fn bump(mut c: C, d: Int):
    c.slot() += d
fn go() -> Int:
    var a = C(1)
    var b = C(10)
    bump(mut a, b.v)
    bump(mut b, a.v)
    a.v * 100 + b.v
print(go())
