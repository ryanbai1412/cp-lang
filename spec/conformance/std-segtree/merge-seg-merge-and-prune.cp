// expect: ok, output "YES YES YES"
// rule: 07.7
use std.segtree.{MergeSeg}
domain K = 0..10
var m = MergeSeg<K>.singleton(K(7)).merge(MergeSeg<K>.singleton(K(3))).merge(MergeSeg<K>.singleton(K(3)))
let a = m.first_alive(x => x.idx > 5)
let b = m.first_alive(x => true)
let c = m.first_alive(x => x.idx > 8)
print(yes(a == Some(K(7))), yes(b == Some(K(7))), yes(c == None))
