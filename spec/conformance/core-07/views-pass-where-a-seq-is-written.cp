// expect: ok, output "5 12 4 12 4"
// rule: 07.1.2
// A std view walks by its `Seq` impl, so `filter`, `map` and `step` views pass where a `Seq<T>` parameter or a `yields T` result is written.
use std.seq.{filter, map}
fn tot(s: Seq<Int>) -> Int:
    var t = 0
    for x in s: t += x
    t
fn total(s: Seq<Int>) -> Int = s.sum()
fn evens(n: Int) -> yields Int = filter(0..n, x => x % 2 == 0)
let a = [1, 2, 3]
print(tot(filter(a, x => x > 1)), tot(map(a, x => x * 2)), tot(a.step(2)), evens(7).sum(), total(a.step(2)))
