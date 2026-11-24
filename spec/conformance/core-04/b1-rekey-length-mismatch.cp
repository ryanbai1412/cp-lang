// expect: R0001
// rule: 04b.1.7
let w = [4, 5, 6].rekey(0..2) // <- here
print(w[0])
