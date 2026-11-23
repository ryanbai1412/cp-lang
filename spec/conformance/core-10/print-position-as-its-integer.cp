// expect: ok, output "1 2 3"
// rule: 10.3.2
// A position of a declared domain prints as `Int(v)`: `1..=n` prints 1..n.
domain V = 1..=3
print([v for v in V])
