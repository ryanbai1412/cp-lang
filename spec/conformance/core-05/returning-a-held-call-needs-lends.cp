// expect: E0603
// rule: 05.5.1 05.5.3
// Returning `filter(xs, p)` over a lent parameter returns a value holding it, so it needs `lends`.
use std.seq.{filter, Filtered}
fn big(xs: List<Int>) -> Filtered<List<Int>, Int> = filter(xs, x => x > 1) // <- here
print(big([1, 2, 3]).count(x => x > 0))
