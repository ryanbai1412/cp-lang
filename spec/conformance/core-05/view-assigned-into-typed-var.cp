// expect: ok, output "3 9"
// rule: 05.7.8
// Lent-ness is not part of a type: a slice of a `[V]Int` assigned into a `var b: [V]Int` declared after its source is kept there (R05.7.8), with no copy.
domain V = 1..=3
let a = [V] v => Int(v) + 1
var b: [V]Int = [V] _ => 0
b = a[..]
print(b.len, b.sum())
