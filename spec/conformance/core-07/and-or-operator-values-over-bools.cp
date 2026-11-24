// expect: ok, output "YES YES NO\nYES NO"
// rule: 07.9.2
// Bare `and` and `or` are operator values over `Bool`, and a closure spelling of either folds the same (adversarial round 46).
let xs = [true, false, true]
let f = (a: Bool, b: Bool) => a or b
print(yes(xs.fold(or)), yes(xs.fold(f)!), yes(xs.fold(and)))
print(yes(f(false, true)), yes([false].fold(or)))
