// expect: E0302
// A domain is not the bitset of its positions: the full set is `Bitset<D>.full()`.
use std.bitset.{Bitset}
domain D = 0..5
fn g(b: Bitset<D>) -> Int = b.len
print(g(D.all)) // <- here
