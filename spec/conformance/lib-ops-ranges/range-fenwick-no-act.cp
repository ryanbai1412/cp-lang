// expect: E0705
// Any constructor without an `act` parameter is asked for a lazy specialization it lacks, not only `Fenwick`'s.
use std.fenwick.{RangeFenwick}
let a = [1, 2, 3]
var f = RangeFenwick(a, act: 1) // <- here
