// expect: ok, output "0"
use std.seq.{maximal}
let a: List<Int> = []
print(a.maximal(state: Counter<Int, Int>(), ok: c => c.len <= 1).len)
