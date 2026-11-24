// expect: E0313
// rule: 02.9.2
let p = { domain V = 0..3; V(1) } // <- here
print(Int(p))
