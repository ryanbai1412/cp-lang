// expect: ok, output "multiset of 3 multiset of 0"
use std.persistent.{PArena, PMultiset}
var ar = PArena<PMultiset<0..10>>()
let s = PMultiset<0..10>.empty().with(mut ar, 7).with(mut ar, 2).with(mut ar, 7)
print(s, PMultiset<0..10>.empty())
