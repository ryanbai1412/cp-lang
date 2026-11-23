// expect: ok, output "0"
use std.seq.{maximal}
let a: List<Int> = []
print(a.maximal(state: Counter<Int, Int>(0), ok: c => c.len <= 1).len)
