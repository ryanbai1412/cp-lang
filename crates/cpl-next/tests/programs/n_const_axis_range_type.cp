// expect: ok, output "3 7 5\n1 3 5 7\n4"
// rule: R04.3.4 R04.4.1
// A constant axis `[0..4]T` is keyed by the range type `0..=3`: its keys walk as `Int`s, a
// literal or open range cuts a piece by `Int`, and an `Int` key is checked into the range.
var u: [0..4]Int = [1, 2, 3, 4]
print(u[..2].sum(), u[2..].sum(), u[1..=2].sum())
for i in u.keys:
    u[i] += i
print(u)
fn last(a: [0..4]Int) -> Int = a[3] - a[0]
print(last([0..4] i => 2 * i - 1) - 2)
