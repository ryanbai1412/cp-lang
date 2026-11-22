// expect: ok, output "YES"
use std.segtree.{MergeSeg}
domain K = 0..10
var m = MergeSeg<K>.singleton(K(3))
print(yes(m.first_alive(x => x.idx > 5) == None))
