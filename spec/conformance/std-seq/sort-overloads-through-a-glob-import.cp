// expect: ok, output "3 2 1\n1 2 3\n3 2 1"
// `use std.seq.*` brings every overload of `sort` and `sorted`, so each call still picks by
// its labels (R06.1.6, decision 624) in the method and the free form.
use std.seq.*
var a = [2, 3, 1]
a.sort(by: x => -x)
print(a)
sort(mut a, cmp: (x, y) => x < y)
print(a)
print(a.sorted(cmp: (x, y) => x > y))
