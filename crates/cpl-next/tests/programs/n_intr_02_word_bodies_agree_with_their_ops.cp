// expect: ok, output "0 64 64 0\n1 0 63 1\n64 0 0 64\n1 63 0 64\n63 0 1 63\n64 0 0 64\n64 2016 2016 2080"
// rule: R12.14.1
// popcount, ctz, clz and bit_width are CP bodies over `W: Word`; the reference interpreter runs
// the bodies and the engines and C++ run their `@fast` ops, so every runner printing the
// same words holds the two together on the edge words (0, 1, -1, the lowest and highest Int as
// words, the all-ones U64) and on every power of two.
use std.math.{Word, popcount, ctz, clz, bit_width}
fn row<W: Word>(x: W):
    print(popcount(x), ctz(x), clz(x), bit_width(x))
row(0)
row(1)
row(-1)
let max = 9223372036854775807
row(U64(1) << 63)
row(max)
row(U64(0) - U64(1))
var (p, c, l, b) = (0, 0, 0, 0)
var w = U64(1)
for _ in 0..64:
    p += popcount(w)
    c += ctz(w)
    l += clz(w)
    b += bit_width(w)
    w = w << 1
print(p, c, l, b)
