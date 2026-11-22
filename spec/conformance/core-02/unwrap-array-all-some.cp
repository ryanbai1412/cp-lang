// expect: ok, output "1"
// rule: 02.4.8
domain V = 1..=3
let depth: [V]?Int = [V] v => Some(1)
let full = depth!
print(full[V(1)])
