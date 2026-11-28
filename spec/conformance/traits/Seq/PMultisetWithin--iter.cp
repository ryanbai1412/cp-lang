// expect: ok, output "2\n7\n7"
// engines: next
use std.persistent.{PArena, PMultiset}
var ar = PArena<PMultiset<0..10>>()
let s = PMultiset<0..10>.empty().with(mut ar, 7).with(mut ar, 2).with(mut ar, 7)
for x in s.within(ar): print(x)
