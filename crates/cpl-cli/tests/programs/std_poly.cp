use std.ntt.{convolve}
use std.poly.{
    compose,
    deriv,
    div_mod,
    eval,
    eval_all,
    exp,
    integral,
    interpolate,
    inv,
    lagrange_at,
    log,
    pow,
    prod_linear,
    sqrt,
    taylor_shift,
}

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
print(yes(sqrt([Z(0), Z(4), Z(1)], 3) == none))
print(yes(sqrt([Z(3), Z(1)], 2) == none))
print(sqrt(List.filled(4, Z(0)), 4)!)

let (dq, dr) = div_mod(a, [Z(3), Z(1), Z(2)])
print(dq)
print(dr)
print(eval(a, Z(5)))
print(prod_linear([Z(1), Z(2), Z(4), Z(7)]))
let sh = taylor_shift(a, Z(3))
print(sh)
print(yes(eval(sh, Z(5)) == eval(a, Z(8))))
let xs = [Z(i * 3 + 1) for i in 0..40]
let ev = eval_all(a, xs)
print(yes((0..xs.len).all(i => ev[i] == eval(a, xs[i]))))
let pn = pow(a, -3, 8)
let pp = pow(a, 3, 8)
let pid = convolve(pp, pn)
print(yes(pid[0].v == 1 and (1..8).all(i => pid[i].v == 0)))
print(compose([Z(3), Z(5), Z(7), Z(11)], [Z(0), Z(2), Z(3)], 6))
print(compose([Z(3), Z(5), Z(7), Z(11)], [Z(4), Z(2), Z(3)], 6))
let ys_l = [eval(a, Z(i)) for i in 0..a.len]
print(yes(lagrange_at(ys_l, Z(100)) == eval(a, Z(100))))
print(yes(lagrange_at(ys_l, Z(3)) == ys_l[3]))
let xs_i = [Z(i * 2 + 1) for i in 0..a.len]
let ys_i = [eval(a, x) for x in xs_i]
print(yes(interpolate(xs_i, ys_i) == a))
let big_p = [Z((i * 7 + 1) % 31) for i in 0..68]
let big_x = [Z(i * 3 + 2) for i in 0..68]
print(yes(interpolate(big_x, eval_all(big_p, big_x)) == big_p))
