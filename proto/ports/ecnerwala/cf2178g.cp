use std.rand.Rng

var rng = Rng.new(48)

@!multitest
let n: Int = read()
let m = 2 * n
var bit: [0..m]U64 = U64(0)
var tot = U64(0)
var out: List<Char> = []
for _ in 0..n:
    let (a0, b0): (Int, Int) = read()
    let a = a0 - 1
    let b = b0 - 1
    var pred = U64(1)
    var x = a
    while x > 0:
        pred ^= bit[x - 1]
        x &= x - 1
    var y = b
    while y > 0:
        pred ^= bit[y - 1]
        y &= y - 1
    if (pred & U64(1)) != U64(0):
        pred ^= rng.next() << 1
    var i = a
    while i < m:
        bit[i] ^= pred
        i |= i + 1
    var j = b
    while j < m:
        bit[j] ^= pred
        j |= j + 1
    tot ^= pred
    out.push(if tot == (tot & U64(1)): '1' else: '0')
print(Str(out))
