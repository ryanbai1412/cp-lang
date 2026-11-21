use std.ntt.{convolve}
use std.poly.{deriv, exp, integral, inv, log, pow, sqrt}

type Z = Mod<998_244_353>

let a: List<Z> = [Z(2), Z(3), Z(5), Z(7), Z(11), Z(13), Z(17), Z(19)]
print(deriv(a))
print(integral(deriv(a)))

let ia = inv(a, 8)
let prod = convolve(a, ia)
print([prod[i] for i in 0..8])

let b: List<Z> = [Z(1), Z(4), Z(9), Z(16), Z(25), Z(36), Z(49), Z(64)]
let lb = log(b, 8)
let eb = exp(lb, 8)
print(eb)

let c: List<Z> = [Z(0), Z(1), Z(2), Z(3), Z(4), Z(5), Z(6), Z(7)]
print(log(exp(c, 8), 8))

let p = pow([Z(0), Z(2), Z(3), Z(5)], 3, 10)
print(p)
print(pow([Z(0), Z(2)], 0, 4))
print(pow([Z(0), Z(2)], 1_000_000_000_000, 5))

let s_in: List<Z> = [Z(0), Z(0), Z(4), Z(12), Z(9), Z(5), Z(7)]
let g = sqrt(s_in, 7)!
let gg = convolve(g, g)
print([gg[i] for i in 0..7])
print(sqrt([Z(0), Z(4), Z(1)], 3) == none)
print(sqrt([Z(3), Z(1)], 2) == none)
print(sqrt(List.filled(4, Z(0)), 4)!)
