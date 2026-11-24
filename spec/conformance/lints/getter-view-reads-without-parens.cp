// expect: ok, output "0\n1\n2"
// rule: 07.1.7
// `.items` on a view is std's `@get items`, an O(1) view, so it needs no `()`.
use std.seq.{Rebased}
let a = [1, 2, 3]
for (k, x) in Rebased(a).items:
    print(k)
