// expect: ok, output "1\n2"
// rule: 04b.1.12 07.4.12
// A column `g[.., c]` keeps the row domain, so its keys are `R` positions, not 0-based offsets (rebuild round 32 FL7).
domain R = 1..=2
domain C = 0..3
let g: [R, C]Int = [R, C] (r, c) => Int(r) * 10 + Int(c)
let col = g[.., C(2)]
print(col.keys.first!)
print([r for (r, x) in col.items if x == 22])
