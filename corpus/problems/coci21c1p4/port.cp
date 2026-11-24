// Source: COCI 2021/2022 Contest 1 #4, Set, https://dmoj.ca/problem/coci21c1p4
// limits: 1000 ms, 512 MB
// Problem: n distinct cards, each a string of k characters from {1,2,3} (k <= 12, n <= 3^k).
//   Count unordered triples of cards that form a set: at every position the three characters
//   are all equal or all different.
// Solution: Three cards form a set iff each position sums to 0 mod 3, so count ordered triples
//   x + y + z = 0 in Z_3^k: take the k-dimensional DFT of the card indicator over Z[w] (w a
//   cube root of unity, stored as two Int arrays), cube every value, sum the real parts and
//   divide by 3^k. Remove the n triples x = y = z (the only degenerate ones), divide by 6.
//   O(k 3^k).
// original: corpus/submissions/4106618.cpp
// Bounds: n: 1..=531441; k: 1..=12

let (n, k): (1..=531441, 1..=12) = read()  // n <= 3^k
let size = 3 ** k

// r + w ω, with ω a cube root of unity (ω² = -1 - ω)
type Eis = (r: Int, w: Int)
fn turn(x: Eis) -> Eis = (r: -x.w, w: x.r - x.w)  // ω x
fn plus(x: Eis, y: Eis, z: Eis) -> Eis = (r: x.r + y.r + z.r, w: x.w + y.w + z.w)

var f: [size]Eis = (r: 0, w: 0)
for _ in 0..n:
    let card: [k]Char = read()
    var code = 0
    for c in card: code = 3 * code + (c - '1')
    f[code].r += 1

// the DFT over Z_3^k, one digit at a time
for d in 0..k:
    let stride = 3 ** d
    for base in (0..size).step(3 * stride):
        for i in base..base + stride:
            let (x, y, z) = (f[i], f[i + stride], f[i + 2 * stride])
            f[i] = plus(x, y, z)
            f[i + stride] = plus(x, turn(y), turn(turn(z)))
            f[i + 2 * stride] = plus(x, turn(turn(y)), turn(z))

// ordered triples x + y + z = 0: the real part of the sum of cubes over 3^k;
// (r + w ω)³ has real part r³ + w³ - 3 r w²
let cubes = sum(x.r * x.r * x.r + x.w * x.w * x.w - 3 * x.r * x.w * x.w for x in f)
print((cubes / size - n) / 6)
