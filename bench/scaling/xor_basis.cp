// fixed: W, o_rank
use std.matrix.{XorBasis, PrefixXorBasis}
let n: Int = read()
var k = 0
var b = XorBasis()
var c = XorBasis()
for i in 0..40:
    b.add(i * 2654435761 % (1 << 50) + 1)
    c.add(i * 40503 % (1 << 45) + 7)
for i in 0..8 * n:
    let x = i * 2654435761 % (1 << 60)
    k += XorBasis().rank // timed: matrix.XorBasis.new
    var t = XorBasis()
    if t.add(x): k += 1 // timed: matrix.XorBasis.add
    k ^= b.reduce(x) // timed: matrix.XorBasis.reduce
    if b.contains(x): k += 1 // timed: matrix.XorBasis.contains
    k ^= b.max_xor(x) // timed: matrix.XorBasis.max_xor
    k ^= b.min_xor(x) // timed: matrix.XorBasis.min_xor
    k += b.rank // timed: matrix.XorBasis.rank
    k += XorBasis.monoid().id.rank // timed: matrix.XorBasis.monoid
for i in 0..n:
    var t = XorBasis()
    k ^= b.kth(i % 1000) ?? 0 // timed: matrix.XorBasis.kth
    k += b.basis_vecs().len // timed: matrix.XorBasis.basis_vecs
    t.merge(c) // timed: matrix.XorBasis.merge
    k += b.intersect(c).rank // timed: matrix.XorBasis.intersect
var p = PrefixXorBasis()
for i in 0..n:
    p.push(i * 2654435761 % (1 << 60)) // timed: matrix.PrefixXorBasis.push
for i in 0..8 * n:
    let r = i % (n / 2)..n
    k += PrefixXorBasis().rank(0..0) // timed: matrix.PrefixXorBasis.new
    k ^= p.max_xor(r) // timed: matrix.PrefixXorBasis.max_xor
    if p.contains(r, i): k += 1 // timed: matrix.PrefixXorBasis.contains
    k += p.rank(r) // timed: matrix.PrefixXorBasis.rank
print(k)
