// expect: ok, output "3 8 YES 2 2 YES"
use std.bitset.{DynBitset}
var d = DynBitset(10)
d.add(8)
d.add(3)
var p = DynBitset(10)
p.add(1)
p.add(3)
var q = DynBitset(10)
q.add(1)
q.add(3)
q.add(8)
print(d.first()!, d.last()!, yes(DynBitset(10).first() == None and d.next(3) == Some(3) and d.next(4) == Some(8) and d.next(9) == None and p.prev(3) == Some(3) and p.prev(2) == Some(1) and p.prev(0) == None), q.count_below(8), q.count_range(1..8), yes(q.nth(1) == Some(3) and q.nth(3) == None))
