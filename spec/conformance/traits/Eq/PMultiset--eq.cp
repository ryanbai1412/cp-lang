// expect: ok, output "YES NO"
use std.persistent.{PArena, PMultiset}
domain K = 0..4
var ar = PArena<PMultiset<K>>()
let e: PMultiset<K> = PMultiset.empty()
let a = e.with(mut ar, K(1)).with(mut ar, K(3))
let b = e.with(mut ar, K(3)).with(mut ar, K(1))
print(yes(a == b), yes(a == e))
