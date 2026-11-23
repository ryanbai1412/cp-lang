// expect: E0301 at 4
// rule: 02.8.1
domain V = 1..=3
domain E = 1..=2
let deg = [V] v => 0
print(deg[E.first!])
