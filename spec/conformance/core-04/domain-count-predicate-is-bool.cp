// expect: E0302
// rule: 04.2.20
domain V = 0..4
print(V.count(v => Int(v))) // <- here
