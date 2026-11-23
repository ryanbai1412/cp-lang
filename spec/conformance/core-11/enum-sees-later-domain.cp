// expect: ok, output "2"
// rule: 11.1.1
// Items hoist: an enum may name a constant domain declared below it (rebuild round 6 H-DECL-ORDER).
enum C:
    At(v: V)
domain V = 0..3
let c = At(2)
match c:
    At(v): print(Int(v))
