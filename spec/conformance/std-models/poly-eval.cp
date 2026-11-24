// expect: ok, output "ok"
// input: "1 300\n"
// model: eval
// model: eval_all
// model: lagrange_at
// model: interpolate
// model: SubproductTree
// model: chirp_z
// model: sample_points_shift
// model: power_sums
// std.poly's evaluation and interpolation against Horner's rule: a random polynomial is the
// truth, its values at points are the input, and every answer is checked by Horner at those
// points (an interpolant of degree < m is unique, so matching all m values is enough). Reads
// `seed n`; prints the first op that differs.
use std.poly.{eval, eval_all, lagrange_at, interpolate, SubproductTree, chirp_z}
use std.poly.{sample_points_shift, power_sums}
use std.mod.{Mod}
use std.rand.{Rng}

type Z = Mod<998_244_353>
const P = 998_244_353

fn horner(a: List<Z>, x: Z) -> Z:
    var v = Z(0)
    for i in (0..a.len).rev: v = v * x + a[i]
    v

fn pw(x: Z, k: Int) -> Z:
    var r = Z(1)
    for _ in 0..k: r *= x
    r

fn rand_list(mut rng: Rng, len: Int) -> List<Z> = [Z(rng.int(0, P - 1)) for _ in 0..len]

// distinct points, sometimes from a tiny range so collisions get redrawn
fn distinct(mut rng: Rng, m: Int) -> List<Z>:
    var seen = Set<Int>()
    var out = List<Z>()
    let hi = if rng.int(0, 1) == 0: 3 * m + 2 else: P - 1
    while out.len < m:
        let v = rng.int(0, hi)
        if v not in seen:
            seen.insert(v)
            out.push(Z(v))
    out

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 7)
    let big = rng.int(0, 4) == 0
    let la = if big: rng.int(33, 50) else: rng.int(0, 8)
    let m = if big: rng.int(33, 45) else: rng.int(0, 8)
    let a = rand_list(mut rng, la)
    if op == 0:
        let x = Z(rng.int(0, P - 1))
        let small = [rng.int(-9, 9) for _ in 0..rng.int(0, 8)]
        let t = rng.int(-3, 3)
        var want = 0
        for i in (0..small.len).rev: want = want * t + small[i]
        if eval(a, x) != horner(a, x): bad = "op {k}: eval({a}, {x})"
        elif eval(small, t) != want: bad = "op {k}: eval({small}, {t})"
    elif op == 1:
        let xs = [Z(rng.int(0, if rng.int(0, 1) == 0: 5 else: P - 1)) for _ in 0..m]
        if eval_all(a, xs) != [horner(a, x) for x in xs]: bad = "op {k}: eval_all({a}, {xs})"
    elif op == 2:
        let f = rand_list(mut rng, m)
        let ys = [horner(f, Z(i)) for i in 0..m]
        let x = Z(if rng.int(0, 1) == 0: rng.int(0, m + 3) else: rng.int(0, P - 1))
        if lagrange_at(ys, x) != horner(f, x): bad = "op {k}: lagrange_at({ys}, {x})"
    elif op == 3:
        let xs = distinct(mut rng, m)
        let ys = rand_list(mut rng, m)
        let f = interpolate(xs, ys)
        if f.len != m or any(horner(f, xs[i]) != ys[i] for i in 0..m): bad = "op {k}: interpolate({xs}, {ys})"
    elif op == 4:
        let xs = distinct(mut rng, m)
        let t = SubproductTree(xs)
        let ys = rand_list(mut rng, m)
        let c = rand_list(mut rng, m)
        let f = t.interpolate(ys)
        let want_t = [sum(c[i] * pw(xs[i], j) for i in 0..m) for j in 0..m]
        if t.eval(a) != [horner(a, x) for x in xs]: bad = "op {k}: SubproductTree({xs}).eval({a})"
        elif f.len != m or any(horner(f, xs[i]) != ys[i] for i in 0..m): bad = "op {k}: SubproductTree({xs}).interpolate({ys})"
        elif t.eval_transposed(c) != want_t: bad = "op {k}: SubproductTree({xs}).eval_transposed({c})"
        elif SubproductTree(xs, keep: m).eval_transposed(c) != want_t: bad = "op {k}: SubproductTree({xs}, keep: {m}).eval_transposed({c})"
    elif op == 5:
        let r = Z(if rng.int(0, 2) == 0: rng.int(0, 3) else: rng.int(0, P - 1))
        if chirp_z(a, r, m) != [horner(a, pw(r, j)) for j in 0..m]: bad = "op {k}: chirp_z({a}, {r}, {m})"
    elif op == 6:
        let f = rand_list(mut rng, la)
        let ys = [horner(f, Z(i)) for i in 0..la]
        let c = Z(if rng.int(0, 1) == 0: rng.int(0, la + 3) else: rng.int(0, P - 1))
        if sample_points_shift(ys, c, m) != [horner(f, c + Z(j)) for j in 0..m]: bad = "op {k}: sample_points_shift({ys}, {c}, {m})"
    else:
        let d = rng.int(0, if big: 40 else: 8)
        let xs = [Z(rng.int(0, if rng.int(0, 1) == 0: 3 else: P - 1)) for _ in 0..m]
        if power_sums(xs, d) != [sum(pw(x, j) for x in xs) for j in 0..=d]: bad = "op {k}: power_sums({xs}, {d})"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
