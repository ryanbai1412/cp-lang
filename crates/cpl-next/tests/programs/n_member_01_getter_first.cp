// expect: ok, output "21 3 21"
// rule: R08.1.4
// `xs.items` without `()` reads the `@get` view, not the trait method of the same name.
use std.seq.{Keyed}
fn total<S: Keyed<Int, Int>>(s: S) -> Int:
    var t = 0
    for (k, x) in s.items: t += k + x
    t
let xs = [5, 6, 7]
var t = 0
for (k, x) in xs.items: t += k + x
print(t, xs.items.len, total(xs))
