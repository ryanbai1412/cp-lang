// expect: ok, output "5"
// rule: R02.10.3
// A fn declared above the `let` its parameter's type ends at, and called below it, reads the
// `let`'s value whatever order the checker visits them in.
fn keep(c: 1..=m) -> Int = c
let m: 1..=10000 = 5
print(keep(5))
