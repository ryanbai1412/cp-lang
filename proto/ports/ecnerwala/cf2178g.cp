// Codeforces 2178 G — Circle Chords Zobrist Parity
//
// Chords (a, b) on 1..=2n are inserted one by one; after each insertion we ask
// whether the crossing graph satisfies the even-crossing parity condition.
//
// Maintain a XOR Fenwick tree over circle points 1..=2n where bit 0 tracks
// endpoint parity and bits 1..64 hold a random 63-bit Zobrist tag. When chord
// (a, b) arrives, `pred = 1 ^ xor_prefix(a.ord) ^ xor_prefix(b.ord)` has bit 0
// set iff the new chord toggles parity; when set, we attach a fresh 63-bit tag
// `rng.next() << 1` and XOR `pred` at both endpoints and into `tot`. All tags
// cancel iff `tot == (tot & 1)`.

use std.rand.Rng

var rng = Rng.new(48)

@!multitest
let n: 1..=200_000 = read()
let m = 2 * n
domain P = 1..=m
let chords: [0..n](a: P, b: P) = read()
var bit: [0..m]U64 = U64(0)

fn xor_prefix(x0: Int) -> U64:
    var res = U64(0)
    var x = x0
    while x > 0:
        res ^= bit[x - 1]
        x &= x - 1
    res

fn xor_add(i0: Int, v: U64):
    var i = i0
    while i < m:
        bit[i] ^= v
        i |= i + 1

var tot = U64(0)
var out: List<Char> = []
for (a, b) in chords:
    var pred = U64(1) ^ xor_prefix(a.ord) ^ xor_prefix(b.ord)
    if (pred & U64(1)) != U64(0): pred ^= rng.next() << 1
    xor_add(a.ord, pred)
    xor_add(b.ord, pred)
    tot ^= pred
    out.push(if tot == (tot & U64(1)): '1' else: '0')

print(Str(out))
