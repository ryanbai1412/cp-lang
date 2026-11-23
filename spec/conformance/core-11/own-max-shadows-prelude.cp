// expect: ok, output "-2\n9"
// rule: 11.2.3
// A file's own `max` shadows the prelude's; the `max` method is unaffected.
fn max(a: Int, b: Int) -> Int = a - b
print(max(3, 5))
let xs = [4, 9, 2]
print(xs.max()!)
