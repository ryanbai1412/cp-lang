use std.fenwick.{XorFenwick}
let n: Int = read()
domain D = 0..n
var f = XorFenwick<D, Int>()
for i in D: f.flip(i, i.to_idx + 1)
print(f.xor(..), f.xor(D(1)..=D(2)), f.prefix(D(2).r))
var h = XorFenwick<D, U64>()
h.flip(D(0), U64(1) << 63)
h.flip(D(3), U64(7))
print(h.xor(..), h.xor(D(1)..), h.xor(D(2)..D(2)))
