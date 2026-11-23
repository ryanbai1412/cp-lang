// expect: ok, output "5 8 3 YES"
// rule: 04.3.3
// `v - w` and `v + k` give an `Int`, `%` goes through `Int(v)`, and a position compares with a literal.
domain V = 0..10
let v = V(7)
let w = V(2)
print(v - w, v + 1, Int(v) % 4, yes(v > 3))
