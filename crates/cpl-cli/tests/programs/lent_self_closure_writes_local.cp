// A closure that writes a caller's local, called from a method whose `mut self`
// is lent: the method's own writes to `self` survive the call.
use std.interval.{SparseIntervalMap}

var z = SparseIntervalMap(-20, 20, 0)
for k in 0..3:
    var seen = 0
    z.assign(k..k + 2, 1, on_old: (rr, _) => seen += rr.len)
    print(seen, z.runs)
