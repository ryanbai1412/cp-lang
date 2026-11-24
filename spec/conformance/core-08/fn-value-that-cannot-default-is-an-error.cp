// expect: E0806
// rule: 08.2.11
// has: "its body does not check with `Int`s"
// A lambda whose unwritten parameter types `Int` does not fit says so; it is never an unknown.
fn ap<X>(op: X) -> X = op
let z = ap((a, b) => a + "x") // <- here
