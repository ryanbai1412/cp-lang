// expect: ok, output "3 7 2 1"
// rule: R02.10.3
// A range type may end at a top-level `let`: in a parameter, a return type and an optional
// array's element, each store checked against the `let`'s value.
let m: 1..=1e4 = 7
domain B = 0..3
fn clamp(c: 1..=m) -> 0..=m = c - 1
var cap: [B]?1..=m = None
cap[B(1)] = 2
print(clamp(4), m, cap[B(1)] ?? 1, cap[B(0)] ?? 1)
