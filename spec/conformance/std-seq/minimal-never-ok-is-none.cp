// expect: ok, output "YES"
use std.seq.{minimal}
print(yes([1, 1].minimal(state: Counter<Int, Int>(0), ok: c => c.len >= 3).map(w => w.map(.len)) == [None, None]))
