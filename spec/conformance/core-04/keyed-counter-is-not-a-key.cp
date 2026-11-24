// expect: E0301
// rule: 04a.11.2
domain V = 1..=3
let a: [V]Int = [V] v => Int(v)
for (c, x) in a.keyed().items:
    print(a[c]) // <- here
