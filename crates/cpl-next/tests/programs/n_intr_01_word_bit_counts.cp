// expect: ok, output "3 64 3 0\n3 64 63 64\n3 0 64 1\n0 64 3 63"
// rule: R12.14.1
// std.math's `@intrinsic` word ops count a 64-bit word's bits; an `Int` counts as its two's
// complement, and a zero word has 64 zeros on either side.
use std.math.{popcount, ctz, clz, bit_width}
print(popcount(7), popcount(-1), popcount(U64(11)), popcount(0))
print(ctz(8), ctz(0), clz(1), clz(0))
print(bit_width(5), bit_width(0), bit_width(-1), bit_width(U64(1)))
print(ctz(U64(1)), ctz(U64(0)), ctz(-8), clz(U64(1)))
