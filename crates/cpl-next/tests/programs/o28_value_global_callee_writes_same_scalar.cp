// expect: ok, output "3 100"
// A scalar global passed by value while the callee writes it: fine, scalars are copies.
var k = 3
fn f(x: Int) -> Int:
    k = 100
    x
print(f(k), k)
