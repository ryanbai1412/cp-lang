// expect: ok, output "12"
// rule: 01.7.3
// A `match` expression may span lines as a call argument; the call continues after its last arm (rebuild round 20 H14).
enum E = A | B
fn f(x: Int, y: Int) -> Int:
    x + y
let e = E.B
print(f(
    match e:
        A => 1
        B => 2
    , 10))
