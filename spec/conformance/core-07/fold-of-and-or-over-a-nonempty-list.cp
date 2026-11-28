// expect: ok, output "NO YES\nYES NO"
// rule: 07.9.2
// A fold of `and` is whether every element holds and one of `or` whether any does, through std's `FromOp<Bool>` impls for the two operator fns.
let xs = [true, false, true]
print(yes(xs.fold(and)), yes(xs.fold(or)))
let ts = [true, true]
let fs = [false, false]
print(yes(ts.fold(and)), yes(fs.fold(or)))
