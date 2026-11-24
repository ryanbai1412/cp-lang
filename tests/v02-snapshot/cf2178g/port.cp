// Source: Codeforces 2178G (deCH OR Dations), https://codeforces.com/contest/2178/problem/G
// Problem: 2 <= n <= 5e5 chords with distinct endpoints on 2n circle points arrive in order. A chain
//   is a set of chords whose consecutive members (by index) cross. For each prefix print 1 if
//   every chord lies in an even number of chains, else 0. Multitest, sum of n <= 5e5.
// Solution: chord i lies in end(i) * start(i) chains (chains ending / starting at it), and
//   end(i) = 1 + the sum of end(j) over earlier crossing chords. Give each chord a 64-bit
//   value: bit 0 is end(i) mod 2, plus a random tag when it is odd, XOR the values of earlier
//   crossing chords (XorFenwick prefix XORs at both endpoints). The XOR of all values cancels
//   tag j iff start(j) is even, so the prefix is tight-knit iff no tag is left. O(n log n).

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
    bit.flip(a, pred)
    bit.flip(b, pred)
    tot ^= pred
    out.push(if tot == (tot & U64(1)): '1' else: '0')

print(Str(out))
