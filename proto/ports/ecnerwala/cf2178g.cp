use std.rand.Rng

var rng = Rng.new(48)

@!multitest
let n: Int = read()
let m = 2 * n
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
for _ in 0..n:
    let (a, b): (Int, Int) = read()
    var pred = U64(1) ^ xor_prefix(a - 1) ^ xor_prefix(b - 1)
    if (pred & U64(1)) != U64(0): pred ^= rng.next() << 1
    xor_add(a - 1, pred)
    xor_add(b - 1, pred)
    tot ^= pred
    out.push(if tot == (tot & U64(1)): '1' else: '0')

print(Str(out))
