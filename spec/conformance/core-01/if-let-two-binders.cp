// expect: ok, output "5"
// rule: 01.7.1
// Each binder in an `if let` chain repeats `let` (rebuild round 16 H-IF-LET-CHAIN).
let a: ?Int = Some(2)
let b: ?Int = Some(3)
if let x = a, let y = b: print(x + y)
