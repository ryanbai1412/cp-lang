// expect: ok, output "3 3 YES 1 8 3 2 8 YES"
use std.bitset.{Bitset}
domain D = 0..10
let b = Bitset<D>.of([D(1), D(3), D(8)])
let e = Bitset<D>.new()
print(b.len, b.count, yes(e.is_empty and not Bitset<D>.full().is_empty), b.first()!, b.nth(2)!, b.rfind(d => Int(d) < 8)!, b.count_range(D(1)..D(8)), [d for d in b.each_rev()][0], yes(b.nth(3) == None))
