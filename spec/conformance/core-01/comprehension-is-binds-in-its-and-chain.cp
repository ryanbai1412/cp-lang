// expect: ok, output "1 3"
// rule: 01.8.2
// `x is pat` in a comprehension's `if` binds inside its own `and` chain (decision 111).
let xs = [Some(1), None, Some(-2), Some(3)]
print([x for x in xs if x is Some(v) and v > 0])
