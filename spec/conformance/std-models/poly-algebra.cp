// expect: ok, output "ok"
// input: "1 300\n"
// model: div_mod
// model: gcd_poly
// model: pow_mod_poly
// model: transposed_mul
// model: bostan_mori
// model: prod_linear
// model: taylor_shift
// model: binomial_transform
// std.poly's polynomial algebra against schoolbook loops: long division, Euclid on long
// division, repeated multiply-and-reduce, the middle product sum, the series p / q one
// coefficient at a time, one linear factor at a time, binomial expansion of (x + c)^i and
// Pascal's triangle. Lengths reach past 32 so the NTT paths run. Reads `seed n`; prints the
// first op that differs.
use std.poly.{div_mod, gcd_poly, pow_mod_poly, transposed_mul, bostan_mori, prod_linear}
use std.poly.{taylor_shift, binomial_transform}
use std.mod.{Mod}
use std.rand.{Rng}

type Z = Mod<998_244_353>
const P = 998_244_353

fn trim(a: List<Z>) -> List<Z>:
    var r = a.clone()
    while not r.is_empty and r.last! == Z(0): r.pop()
    r

fn mul(a: List<Z>, b: List<Z>) -> List<Z>:
    if a.is_empty or b.is_empty: return []
    var c = List.filled(a.len + b.len - 1, Z(0))
    for i in 0..a.len:
        for j in 0..b.len: c[i + j] += a[i] * b[j]
    c

fn long_div(a: List<Z>, b: List<Z>) -> (List<Z>, List<Z>):
    let (x, y) = (trim(a), trim(b))
    if x.len < y.len: return ([], x)
    var r = x.clone()
    var q = List.filled(x.len - y.len + 1, Z(0))
    for i in (0..q.len).rev:
        let f = r[i + y.len - 1] / y.last!
        q[i] = f
        for j in 0..y.len: r[i + j] -= f * y[j]
    (q, trim(r))

fn euclid(a: List<Z>, b: List<Z>) -> List<Z>:
    var (x, y) = (trim(a), trim(b))
    while not y.is_empty: (x, y) = (y, long_div(x, y).1)
    if x.is_empty: return x
    let l = x.last!
    [c / l for c in x]

fn rand_list(mut rng: Rng, len: Int) -> List<Z> = [Z(rng.int(0, P - 1)) for _ in 0..len]

var pascal = [[Z(1)]]
for i in 1..60:
    pascal.push([if j == 0 or j == i: Z(1) else: pascal[i - 1][j - 1] + pascal[i - 1][j] for j in 0..=i])

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 7)
    let big = rng.int(0, 4) == 0
    let la = if big: rng.int(33, 50) else: rng.int(0, 8)
    let lb = if big: rng.int(33, 45) else: rng.int(0, 6)
    var a = rand_list(mut rng, la)
    var b = rand_list(mut rng, lb)
    for i in 0..rng.int(0, 2):
        if a.len > 0: a[a.len - 1 - min(i, a.len - 1)] = Z(0)
    if op == 0:
        if trim(b).is_empty: b = b + [Z(rng.int(1, 9))]
        let (q, r) = long_div(a, b)
        let got = div_mod(a, b)
        if got.q != q or got.r != r: bad = "op {k}: div_mod({a}, {b})"
    elif op == 1:
        // a shared factor makes the gcd nontrivial
        let c = rand_list(mut rng, rng.int(0, 4))
        let (x, y) = if rng.int(0, 1) == 0: (mul(a, c), mul(b, c)) else: (a, b)
        if gcd_poly(x, y) != euclid(x, y): bad = "op {k}: gcd_poly({x}, {y})"
    elif op == 2:
        if trim(b).is_empty: b = b + [Z(rng.int(1, 9))]
        let e = if rng.int(0, 2) == 0: rng.int(0, 1_000_000_000) else: rng.int(0, 12)
        let m = trim(b)
        var (acc, base, t) = ([Z(1)], long_div(a, m).1, e)
        while t > 0:
            if t % 2 == 1: acc = long_div(mul(acc, base), m).1
            base = long_div(mul(base, base), m).1
            t /= 2
        acc = long_div(acc, m).1
        let want = [if i < acc.len: acc[i] else: Z(0) for i in 0..m.len - 1]
        if pow_mod_poly(a, e, b) != want: bad = "op {k}: pow_mod_poly({a}, {e}, {b})"
    elif op == 3:
        let want = if b.is_empty: List<Z>() else: [sum(a[i + j] * b[j] for j in 0..b.len) for i in 0..max(0, a.len - b.len + 1)]
        if transposed_mul(a, b) != want: bad = "op {k}: transposed_mul({a}, {b})"
    elif op == 4:
        if b.is_empty: b = [Z(1)]
        if b[0] == Z(0): b[0] = Z(1)
        let t = rng.int(0, 120)
        var s = List.filled(t + 1, Z(0))
        for i in 0..=t:
            var v = if i < a.len: a[i] else: Z(0)
            for j in 1..min(i + 1, b.len): v -= b[j] * s[i - j]
            s[i] = v / b[0]
        if bostan_mori(a, b, t) != s[t]: bad = "op {k}: bostan_mori({a}, {b}, {t})"
    elif op == 5:
        var want = [Z(1)]
        for r in a: want = mul(want, [Z(0) - r, Z(1)])
        if prod_linear(a) != want: bad = "op {k}: prod_linear({a})"
    elif op == 6:
        let c = Z(rng.int(0, P - 1))
        var want = List.filled(a.len, Z(0))
        for i in 0..a.len:
            var cp = Z(1)
            for j in (0..=i).rev:
                want[j] += a[i] * pascal[i][j] * cp
                cp *= c
        if taylor_shift(a, c) != want: bad = "op {k}: taylor_shift({a}, {c})"
    else:
        let want = [sum(pascal[i][j] * a[j] for j in 0..=i) for i in 0..a.len]
        let back = [sum((if (i - j) % 2 == 0: pascal[i][j] else: Z(0) - pascal[i][j]) * a[j] for j in 0..=i) for i in 0..a.len]
        if binomial_transform(a) != want: bad = "op {k}: binomial_transform({a})"
        elif binomial_transform(a, inverse: true) != back: bad = "op {k}: binomial_transform({a}, inverse: true)"
        elif binomial_transform(want, inverse: true) != a: bad = "op {k}: binomial_transform round trip {a}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
