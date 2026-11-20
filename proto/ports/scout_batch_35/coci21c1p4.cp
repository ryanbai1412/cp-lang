// COCI 2021/22 #1 Set: n distinct cards, each a word of k digits 1..=3.
// Three cards form a set when in every position their digits are all equal
// or all different. Count the sets.
//
// As vectors over Z_3 a set is three distinct cards with x + y + z = 0.
// The number of ordered triples with sum 0 is the Fourier sum
// (1 / 3^k) * sum over characters of fhat^3, with fhat in Z[w], w^3 = 1.
// Only x = y = z is a non-set among them, n times; each set is counted 6 times.
// original: corpus/submissions/4106618.cpp
let (n, k): (1..=531441, 1..=12) = read()
let cards: [0..n]Str = read()
let size = 3 ** k

type Eis = (re: Int, w: Int)                    // re + w * omega, omega^2 = -1 - omega
fn add(x: Eis, y: Eis) -> Eis: (re: x.re + y.re, w: x.w + y.w)
fn mul(x: Eis, y: Eis) -> Eis:
    (re: x.re * y.re - x.w * y.w, w: x.re * y.w + x.w * y.re - x.w * y.w)
fn turn(x: Eis) -> Eis: (re: -x.w, w: x.re - x.w)   // times omega

var f: [0..size]Eis = (re: 0, w: 0)
for card in cards:
    var code = 0
    for ch in card: code = code * 3 + (ch - '1')
    f[code].re += 1

var stride = 1
while stride < size:
    for base in (0..size).step(3 * stride):
        for i in base..base + stride:
            let (x, y, z) = (f[i], f[i + stride], f[i + 2 * stride])
            f[i] = add(x, add(y, z))
            f[i + stride] = add(x, add(turn(y), turn(turn(z))))
            f[i + 2 * stride] = add(x, add(turn(turn(y)), turn(z)))
    stride *= 3

let triples = sum(I128(mul(x, mul(x, x)).re) for x in f) / size
print((triples - n) / 6)
