// expect: ok, output "1 2\n0"
use std.seq.{merge_sorted}
let e: List<Int> = []
print(merge_sorted(e, [1, 2]))
print(merge_sorted(e, e).len)
