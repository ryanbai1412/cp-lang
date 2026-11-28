// expect: ok, output "YES\nNO"
// A traced run shows a `Bool`, which is not `Show`, as `true` or `false`, through std's trace fns.
fn odd(n: Int) -> Bool = n % 2 == 1
var b = odd(3)
print(yes(b))
b = odd(4)
print(yes(b))
