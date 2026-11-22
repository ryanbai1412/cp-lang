// expect: E0301 at 5
// rule: 04b.1.3
domain V = 1..=3
domain E = 1..=3
let a = [V] v => 1
let b = [E] e => 1
for (x, y) in zip(a, b): print(x + y)
