// `U64` beside an `Int`: shifts, comparisons, big literals and bit counts.
let k: Int = read()
let top: U64 = 1 << (k + 1)
let high: U64 = 1 << 63
print(top, high, high >> k)
let m: Int = read()
let c: U64 = 5
let big: U64 = 18446744073709551615
print(yes(m < c), yes(c > m), yes(big > m), yes(big == m), yes(c >= 5), yes(5 == c), yes(big > 7))
print(big, yes(big == 0xFFFF_FFFF_FFFF_FFFF))
print(popcount(big), ctz(high), bit_width(big), popcount(255), bit_width(-1))
