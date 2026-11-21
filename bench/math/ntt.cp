// Matched benchmark: convolve<998244353> on two length-262144 polynomials.
use std.ntt.{convolve}

type Z = Mod<998_244_353>

let n = 262_144
var a = List.filled(n, Z(0))
var b = List.filled(n, Z(0))
var s = 123456789
unchecked:
    for i in 0..n:
        s = (s * 1103515245 + 12345) & 2_147_483_647
        a[i] = Z(s)
        s = (s * 1103515245 + 12345) & 2_147_483_647
        b[i] = Z(s)

let c = convolve(a, b)
var acc = Z(0)
unchecked:
    for i in 0..c.len: acc = acc + c[i] * Z(i + 1)
print(acc.v)
