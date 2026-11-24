// expect: ok, output "4 5 2 4 6 8 10\n1 3 5 2 3 4 5 6\n1 2|-1 -2 -3 -4 -5"
// rule: 07.1.2
// A `filter` or `map` view shows its elements in walk order, free or method form, in `print` and in an interpolation.
use std.seq.{filter, map}
let a = [1, 2, 3, 4, 5]
print(filter(1..=5, x => x > 3), map(1..=5, x => x * 2))
print(a.filter(x => x % 2 == 1), a.map(x => x + 1))
print("{filter(1..=5, x => x < 3)}|{map(1..=5, x => -x)}")
