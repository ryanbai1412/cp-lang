// expect: ok, output "2 7 7"
use std.persistent.{PArena, PMultiset}
var ar = PArena<PMultiset<0..10>>()
let s = PMultiset<0..10>.empty().with(mut ar, 7).with(mut ar, 2).with(mut ar, 7)
print(s.within(ar))
