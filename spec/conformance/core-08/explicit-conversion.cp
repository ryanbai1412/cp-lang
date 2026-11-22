// expect: ok, output "YES"
// rule: 08.5.2
fn f(x: F64) -> F64 = x
let n = 3
print(yes(f(F64(n)) == 3.0))
