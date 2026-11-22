// expect: ok, output "YES"
use std.segtree.{MergeSeg}
var m = MergeSeg<0..10>.singleton(3)
print(yes(m.first_alive(x => x > 5) == None))
