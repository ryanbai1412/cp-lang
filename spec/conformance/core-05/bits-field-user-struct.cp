// expect: ok, output "2 1 3 2 YES"
// rule: 05.10.15 05.10.16
// A user struct over a domain marks a `List<U64>` field `@bits`, as std's `Bitset<D>` does; a Bitset's count and set ops read its words.
use std.bitset.{Bitset}
struct Row<D>:
    @bits w: List<U64>

    fn new() -> Row<D> = Row(List.filled((D.count + 63) / 64, U64(0)))

domain D = 0..100
let r = Row<D>.new()
var a = Bitset<D>()
a.add(D(3))
a.add(D(70))
var b = Bitset<D>()
b.add(D(70))
b.add(D(5))
print(r.w.len, (a & b).len, (a | b).len, (a ^ b).len, yes(Bitset<D>().is_empty))
