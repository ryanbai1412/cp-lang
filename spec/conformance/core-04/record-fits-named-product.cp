// expect: ok, output "2 2"
// rule: 04.2.10
// A record whose names match a product domain's named factors, in order, is a position in it, as a `let` and as an argument.
domain C = 0..3
domain State = (lo: C, hi: C)
fn f(s: State) -> Int = s.lo.ord * 10 + s.hi.ord
let x: C = C.first!
let y: C = C.last!
let s: State = (lo: x, hi: y)
print(f(s), f((lo: x, hi: y)))
