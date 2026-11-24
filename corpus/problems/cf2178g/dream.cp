// Codeforces 2178 G — Circle Chords Zobrist Parity
//
// Chords (a, b) with distinct endpoints on 1..=2n are inserted one by one;
// after each insertion we output '1' iff the current chord set is valid (all
// active parity-tagged chords cancel out under crossing XOR), else '0'.
//
// Maintain `bit: XorFenwick<P, U64>` over circle points `P = 1..=2n`, where
// each inserted chord stores its 64-bit mask `pred` at both endpoints `a` and
// `b`. Because endpoints are distinct, `bit.xor(..a) ^ bit.xor(..b)` is the
// XOR of `pred` over all earlier chords with exactly one endpoint strictly
// between `a` and `b` — i.e., all earlier chords crossing `(a, b)`.
//
// Bit 0 of `pred = 1 ^ bit.xor(..a) ^ bit.xor(..b)` is 1 iff `(a, b)` toggles
// the parity of its crossing set; whenever bit 0 is set, we mix in a fresh
// 63-bit Zobrist tag `rng.next() << 1` so non-cancelling chords cannot collide.
// XORing `pred` at `a` and `b` and into `tot` makes all Zobrist tags cancel
// (leaving `tot == (tot & 1)`) iff the current prefix is valid.

use std.fenwick.{XorFenwick}
use std.rand.Rng

var rng = Rng.new(48)

@!multitest
let n: 2..=500_000 = read()
domain P = 1..=2 * n
let chords: [0..n](a: P, b: P) = read()

var bit = XorFenwick<P, U64>()
var tot = U64(0)
var out: List<Char> = []
for (a, b) in chords:
    var pred = U64(1) ^ bit.xor(..a) ^ bit.xor(..b)
    if (pred & U64(1)) != U64(0): pred ^= rng.next() << 1
    bit.add(a, pred)
    bit.add(b, pred)
    tot ^= pred
    out.push(if tot == (tot & U64(1)): '1' else: '0')

print(Str(out))
