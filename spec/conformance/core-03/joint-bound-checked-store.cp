// expect: ok, output "150000"
// rule: 03.1.1
// input: "300 500\n"
// A joint bound `n * m <= 2e5` is a checked store, and the domain over it gets its width (rebuild round 16 H-JOINT-BOUND).
let (n, m): (1..=2e5, 1..=2e5) = read()
let nm: 1..=2e5 = n * m
domain Val = 1..=nm
print(Val.count)
