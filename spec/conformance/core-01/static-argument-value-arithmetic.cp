// expect: ok, output "4"
// rule: 01.3.4
// A static parameter read as a value (`P - a.v` in `M<P>`) is an `Int`; it needs no `Sub` bound.
struct M<P>(v: 0..P)
fn neg<P>(a: M<P>) -> Int = P - a.v
print(neg(M<7>(3)))
