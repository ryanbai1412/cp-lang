// expect: ok, output "ok"
// input: "1 300\n"
// model: deriv
// model: integral
// model: inv
// model: log
// model: exp
// model: sqrt
// model: compose
// model: compose_inv
// model: Poly
// std.poly's formal power series against the coefficient recurrences: inv, log and exp from
// a * b, a' = l' a and e' = a' e solved one coefficient at a time, pow by repeated truncated
// products, sqrt and compose_inv by squaring and composing back, compose by Horner on truncated
// products, and Poly's methods against the same loops. Reads `seed n`; prints the first op that
// differs.
use std.poly.{deriv, integral, inv, log, exp, pow, sqrt, compose, compose_inv, Poly}
use std.math.{pow_mod}
use std.mod.{Mod}
use std.rand.{Rng}

type Z = Mod<998_244_353>
const P = 998_244_353

fn mul(a: List<Z>, b: List<Z>, n: Int) -> List<Z>:
    var c = List.filled(n, Z(0))
    for i in 0..min(a.len, n):
        for j in 0..min(b.len, n - i): c[i + j] += a[i] * b[j]
    c

fn at(a: List<Z>, i: Int) -> Z = if i < a.len: a[i] else: Z(0)

fn inv_b(a: List<Z>, n: Int) -> List<Z>:
    var b = List.filled(n, Z(0))
    for k in 0..n:
        var s = if k == 0: Z(1) else: Z(0)
        for j in 1..=k: s -= at(a, j) * b[k - j]
        b[k] = s / a[0]
    b

fn log_b(a: List<Z>, n: Int) -> List<Z>:
    var l = List.filled(n, Z(0))
    for k in 1..n:
        var s = Z(k) * at(a, k)
        for j in 1..k: s -= Z(j) * l[j] * at(a, k - j)
        l[k] = s / Z(k)
    l

fn exp_b(a: List<Z>, n: Int) -> List<Z>:
    var e = List.filled(n, Z(0))
    if n > 0: e[0] = Z(1)
    for k in 1..n:
        var s = Z(0)
        for j in 1..=k: s += Z(j) * at(a, j) * e[k - j]
        e[k] = s / Z(k)
    e

fn pow_b(a: List<Z>, k: Int, n: Int) -> List<Z>:
    var (r, b, e) = (mul([Z(1)], [Z(1)], n), if k < 0: inv_b(a, n) else: mul(a, [Z(1)], n), abs(k))
    while e > 0:
        if e % 2 == 1: r = mul(r, b, n)
        b = mul(b, b, n)
        e /= 2
    r

fn compose_b(f: List<Z>, g: List<Z>, n: Int) -> List<Z>:
    var r = List.filled(n, Z(0))
    for i in (0..f.len).rev:
        r = mul(r, g, n)
        if n > 0: r[0] += f[i]
    r

fn rand_list(mut rng: Rng, len: Int) -> List<Z> = [Z(rng.int(0, P - 1)) for _ in 0..len]

// `got` squares to `a` mod x^n with its lowest nonzero coefficient the smaller root, or is None
// exactly when the lowest nonzero coefficient of `a` below x^n sits at an odd index or is a
// non-residue.
fn sqrt_ok(a: List<Z>, n: Int, got: ?List<Z>) -> Bool:
    var t = 0
    while t < n and at(a, t) == Z(0): t += 1
    if t == n: return got == Some(List.filled(n, Z(0)))
    let v = at(a, t).v
    if t % 2 == 1 or pow_mod(v, (P - 1) / 2, P) != 1: return got == None
    if got == None: return false
    let g = got!
    if g.len != n or mul(g, g, n) != mul(a, [Z(1)], n): return false
    var m = 0
    while g[m] == Z(0): m += 1
    let s = g[m].v
    m == t / 2 and s <= P - s

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 9)
    let len = if rng.int(0, 4) == 0: rng.int(20, 45) else: rng.int(0, 8)
    let m = if rng.int(0, 4) == 0: rng.int(20, 45) else: rng.int(0, 8)
    var a = rand_list(mut rng, len)
    if op == 0:
        let d = [Z(i) * a[i] for i in 1..a.len]
        let it = if a.is_empty: List<Z>() else: [Z(0)] + [a[i] / Z(i + 1) for i in 0..a.len]
        if deriv(a) != d: bad = "op {k}: deriv({a})"
        elif integral(a) != it or deriv(integral(a)) != a: bad = "op {k}: integral({a})"
    elif op == 1:
        if a.is_empty: a = [Z(1)]
        if a[0] == Z(0): a[0] = Z(5)
        if inv(a, m) != inv_b(a, m): bad = "op {k}: inv({a}, {m})"
    elif op == 2:
        if a.is_empty: a = [Z(1)]
        a[0] = Z(1)
        if log(a, m) != log_b(a, m): bad = "op {k}: log({a}, {m})"
    elif op == 3:
        if not a.is_empty: a[0] = Z(0)
        if exp(a, m) != exp_b(a, m): bad = "op {k}: exp({a}, {m})"
        elif m > 0 and log(exp(a, m), m) != mul(a, [Z(1)], m): bad = "op {k}: log(exp({a}, {m}))"
    elif op == 4:
        let e = if rng.int(0, 3) == 0: rng.int(0, 1_000_000_000_000) else: rng.int(-5, 8)
        for i in 0..min(a.len, rng.int(0, 2)): a[i] = Z(0)
        if e < 0 and (a.is_empty or a[0] == Z(0)): a = [Z(3)] + a
        if pow(a, e, m) != pow_b(a, e, m): bad = "op {k}: pow({a}, {e}, {m})"
    elif op == 5:
        let h = rand_list(mut rng, rng.int(0, 6))
        var sq = if rng.int(0, 1) == 0: mul(h, h, 2 * h.len) else: a
        for i in 0..min(sq.len, rng.int(0, 3)): sq[i] = Z(0)
        if not sqrt_ok(sq, m, sqrt(sq, m)): bad = "op {k}: sqrt({sq}, {m})"
    elif op == 6:
        let g = rand_list(mut rng, rng.int(0, 6))
        if compose(a, g, m) != compose_b(a, g, m): bad = "op {k}: compose({a}, {g}, {m})"
    elif op == 7:
        var f = [Z(0), Z(rng.int(1, P - 1))] + a
        let g = compose_inv(f, m)
        var x = List.filled(m, Z(0))
        if m > 1: x[1] = Z(1)
        if g.len != m or (m > 0 and g[0] != Z(0)) or compose_b(f, g, m) != x: bad = "op {k}: compose_inv({f}, {m})"
    elif op == 8:
        let b = rand_list(mut rng, rng.int(0, 6))
        let (p, q) = (Poly<Z>.new(a), Poly<Z>(b))
        var deg: ?Int = None
        for i in 0..a.len:
            if a[i] != Z(0): deg = Some(i)
        let plus = [at(a, i) + at(b, i) for i in 0..max(a.len, b.len)]
        let dif = [at(a, i) - at(b, i) for i in 0..max(a.len, b.len)]
        if p.len != a.len or p.deg() != deg or p.is_zero() != (deg == None) or p.c != a: bad = "op {k}: Poly({a}) len/deg"
        elif any(p.at(i) != at(a, i) for i in 0..a.len + 3): bad = "op {k}: Poly({a}).at"
        elif (p + q).c != plus or (p - q).c != dif or (-p).c != [Z(0) - x for x in a]: bad = "op {k}: Poly({a}) +- {b}"
        elif (p * q).c != (if a.is_empty or b.is_empty: List<Z>() else: mul(a, b, a.len + b.len - 1)): bad = "op {k}: Poly({a}) * {b}"
        elif p.deriv().c != deriv(a) or p.integral().c != integral(a): bad = "op {k}: Poly({a}).deriv/integral"
        elif p.truncated(m).c != mul(a, [Z(1)], m): bad = "op {k}: Poly({a}).truncated({m})"
        elif p.transposed_mul(q).c != [sum(a[i + j] * b[j] for j in 0..b.len) for i in 0..(if b.is_empty: 0 else: max(0, a.len - b.len + 1))]: bad = "op {k}: Poly({a}).transposed_mul({b})"
        elif Poly<Z>.zero.c != List<Z>() or Poly<Z>.one.c != [Z(1)] or Poly<Z>.x.c != [Z(0), Z(1)] or Poly<Z>.from_int(len).c != [Z(len)]: bad = "op {k}: Poly constants"
        else:
            let x = Z(rng.int(0, P - 1))
            var (v, pw) = (Z(0), Z(1))
            for c in a:
                v += c * pw
                pw *= x
            var r = p.clone()
            let i = rng.int(0, 10)
            r.set(i, x)
            var want = a.clone()
            while want.len <= i: want.push(Z(0))
            want[i] = x
            if p.eval(x) != v or a.eval(x) != v: bad = "op {k}: Poly({a}).eval({x})"
            elif r.c != want: bad = "op {k}: Poly({a}).set({i}, {x})"
            elif p.shift(x).c != compose_b(a, [x, Z(1)], a.len): bad = "op {k}: Poly({a}).shift({x})"
    else:
        var u = if a.is_empty: [Z(1)] else: a
        if u[0] == Z(0): u[0] = Z(2)
        let p = Poly<Z>(u)
        var l = u.clone()
        l[0] = Z(1)
        var e = u.clone()
        e[0] = Z(0)
        let kk = rng.int(-3, 6)
        let h = rand_list(mut rng, rng.int(1, 5))
        if p.inv(m).c != inv_b(u, m): bad = "op {k}: Poly({u}).inv({m})"
        elif Poly<Z>(l).log(m).c != log_b(l, m): bad = "op {k}: Poly({l}).log({m})"
        elif Poly<Z>(e).exp(m).c != exp_b(e, m): bad = "op {k}: Poly({e}).exp({m})"
        elif p.pow(kk, m).c != pow_b(u, kk, m): bad = "op {k}: Poly({u}).pow({kk}, {m})"
        elif not sqrt_ok(mul(h, h, 2 * h.len), m, Poly<Z>(mul(h, h, 2 * h.len)).sqrt(m).map(s => s.c)): bad = "op {k}: Poly({h}^2).sqrt({m})"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
