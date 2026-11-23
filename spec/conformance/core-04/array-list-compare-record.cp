// expect: ok, output "YES YES NO"
// rule: 04.2.18
// An array and a list literal of the same cells compare as lists, also inside a record.
domain V = 0..2
let a: [V]Bool = [true, false]
let r = (n: 1, a: a)
print(yes(a == [true, false]), yes(r == (n: 1, a: [true, false])), yes(r == (n: 1, a: [true, true])))
