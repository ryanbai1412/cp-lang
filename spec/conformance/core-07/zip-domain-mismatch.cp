// expect: E0301 at 5
// rule: 07.4.10
domain V = 1..=2
domain E = 1..=2
let a = [V] v => 1
let b = [E] e => 2
let z = zip(a, b)
