// expect: ok, output "ok"
// input: "1 300\n"
// model: Mat
// model: MatPow
// std.matrix's dense Mat over Mod<7> (small enough that singular matrices are common) against a
// plain List<List<_>>: products and powers by the triple loop, det, adjugate and char_poly by
// cofactor expansion, the Pfaffian by expansion along row 0, rref and solve_space by a textbook
// Gauss-Jordan, det_mod by cofactor expansion over Int, and the min-plus semiring by its own triple
// loop. MatPow is checked against repeated mul_vec. Reads `seed n`; prints the first op that
// differs.
use std.matrix.{Mat, MatPow}
use std.algebra.{Semiring}
use std.mod.{Mod}
use std.rand.{Rng}

type F = Mod<7>

fn rand_grid(mut rng: Rng, r: Int, c: Int) -> List<List<F>>:
    var g = [[F(if rng.int(0, 2) == 0: 0 else: rng.int(0, 6)) for _ in 0..c] for _ in 0..r]
    // a repeated or scaled row makes rank deficiency common
    if r >= 2 and rng.int(0, 2) == 0:
        let (i, j, s) = (rng.int(0, r - 1), rng.int(0, r - 1), F(rng.int(0, 6)))
        g[i] = [s * x for x in g[j]]
    g

fn mm(a: List<List<F>>, b: List<List<F>>, c: Int) -> List<List<F>>:
    [[sum(a[i][k] * b[k][j] for k in 0..b.len) for j in 0..c] for i in 0..a.len]

fn ident(n: Int) -> List<List<F>> = [[F(if i == j: 1 else: 0) for j in 0..n] for i in 0..n]

fn minor<T>(a: List<List<T>>, r: Int, c: Int) -> List<List<T>>:
    [[a[i][j] for j in 0..a.len if j != c] for i in 0..a.len if i != r]

fn det_b(a: List<List<F>>) -> F:
    if a.is_empty: return F(1)
    var s = F(0)
    for j in 0..a.len:
        let t = a[0][j] * det_b(minor(a, 0, j))
        s = if j % 2 == 0: s + t else: s - t
    s

fn det_int(a: List<List<Int>>) -> Int:
    if a.is_empty: return 1
    var s = 0
    for j in 0..a.len:
        let t = a[0][j] * det_int(minor(a, 0, j))
        s = if j % 2 == 0: s + t else: s - t
    s

fn pf_b(a: List<List<F>>) -> F:
    if a.is_empty: return F(1)
    var s = F(0)
    for j in 1..a.len:
        let rest = [i for i in 1..a.len if i != j]
        let sub = [[a[x][y] for y in rest] for x in rest]
        let t = a[0][j] * pf_b(sub)
        s = if j % 2 == 1: s + t else: s - t
    s

fn rref_b(g: List<List<F>>, cols: Int) -> (List<List<F>>, List<Int>):
    var a = g.clone()
    var piv = List<Int>()
    for c in 0..cols:
        let r = piv.len
        var p = r
        while p < a.len and a[p][c] == F(0): p += 1
        if p == a.len: continue
        (a[r], a[p]) = (a[p], a[r])
        let f = a[r][c].inv()
        a[r] = [x * f for x in a[r]]
        for i in 0..a.len:
            if i != r and a[i][c] != F(0):
                let t = a[i][c]
                a[i] = [a[i][j] - t * a[r][j] for j in 0..cols]
        piv.push(c)
    (a, piv)

fn mk(g: List<List<F>>, r: Int, c: Int) -> Mat<F> = Mat<F>.from_flat(r, c, [g[i][j] for i in 0..r, j in 0..c])

fn grid(m: Mat<F>) -> List<List<F>> = [[m.get(i, j) for j in 0..m.cols] for i in 0..m.rows]

fn mp(a: List<List<?Int>>, b: List<List<?Int>>, c: Int) -> List<List<?Int>>:
    let none: ?Int = None
    var out = [List.filled(c, none) for _ in 0..a.len]
    for i in 0..a.len:
        for j in 0..c:
            for k in 0..b.len:
                if a[i][k] != None and b[k][j] != None:
                    let v = a[i][k]! + b[k][j]!
                    if out[i][j] == None or v < out[i][j]!: out[i][j] = Some(v)
    out

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 9)
    let (r, c) = (rng.int(0, 5), rng.int(0, 5))
    let s = rng.int(0, 5)
    let g = rand_grid(mut rng, r, c)
    let q = rand_grid(mut rng, s, s)
    let m = mk(g, r, c)
    let sq = mk(q, s, s)
    if op == 0:
        let h = rand_grid(mut rng, r, c)
        let flat = [g[i][j] for i in 0..r, j in 0..c]
        var e = Mat<F>.zeros(r, c)
        for i in 0..r:
            for j in 0..c:
                if rng.int(0, 1) == 0: e.set(i, j, g[i][j])
                else: e.at(i, j) += g[i][j]
        let f = F(rng.int(0, 6))
        let of = Mat<F>.of(g)
        if of.rows != r or of.cols != (if r == 0: 0 else: c) or grid(of) != g or (r > 0 and of != m): bad = "op {k}: Mat.of {g}"
        elif Mat<F>.from_flat(r, c, flat) != e or e.rows != r or e.cols != c or grid(e) != g or Mat<F>.from_rows(g) != of: bad = "op {k}: from_flat/from_rows/set/at {g}"
        elif grid(Mat<F>.filled(r, c, f)) != [[f for _ in 0..c] for _ in 0..r] or grid(Mat<F>.zeros(r, c)) != [[F(0) for _ in 0..c] for _ in 0..r]: bad = "op {k}: filled/zeros {r} {c}"
        elif grid(Mat<F>.identity(s)) != ident(s): bad = "op {k}: identity({s})"
        elif grid(m.transpose()) != [[g[i][j] for i in 0..r] for j in 0..m.cols]: bad = "op {k}: transpose {g}"
        elif grid(m + mk(h, r, c)) != [[g[i][j] + h[i][j] for j in 0..c] for i in 0..r]: bad = "op {k}: {g} + {h}"
        elif grid(m - mk(h, r, c)) != [[g[i][j] - h[i][j] for j in 0..c] for i in 0..r]: bad = "op {k}: {g} - {h}"
        elif (m == mk(h, r, c)) != (g == h): bad = "op {k}: {g} == {h}"
    elif op == 1:
        let h = rand_grid(mut rng, c, s)
        let v = [F(rng.int(0, 6)) for _ in 0..c]
        let mono = Mat<F>.product(s)
        if grid(m * mk(h, c, s)) != mm(g, h, s): bad = "op {k}: {g} * {h}"
        elif m.mul_vec(v) != [sum(g[i][j] * v[j] for j in 0..c) for i in 0..r]: bad = "op {k}: {g}.mul_vec({v})"
        elif grid(mono.id) != ident(s) or mono.op(mono.id, sq) != sq or grid(mono.op(sq, sq)) != mm(q, q, s): bad = "op {k}: Mat.product({s}) on {q}"
    elif op == 2:
        let e = rng.int(0, 12)
        var want = ident(s)
        for _ in 0..e: want = mm(want, q, s)
        let p = MatPow(sq, rng.int(e, 40))
        var v = [F(rng.int(0, 6)) for _ in 0..s]
        let got = p.apply_pow(e, v)
        for _ in 0..e: v = sq.mul_vec(v)
        if grid(sq ** e) != want: bad = "op {k}: {q} ** {e}"
        elif got != v: bad = "op {k}: MatPow({q}).apply_pow({e})"
    elif op == 3:
        let d = det_b(q)
        let adj = [[if (i + j) % 2 == 0: det_b(minor(q, j, i)) else: F(0) - det_b(minor(q, j, i)) for j in 0..s] for i in 0..s]
        if sq.det() != d: bad = "op {k}: det {q}"
        elif grid(sq.adjugate()) != adj: bad = "op {k}: adjugate {q}"
        elif (sq.inverse() == None) != (d == F(0)): bad = "op {k}: inverse exists {q}"
        elif d != F(0) and grid(sq * sq.inverse()!) != ident(s): bad = "op {k}: inverse {q}"
    elif op == 4:
        let (e, piv) = rref_b(g, c)
        let (mat: got, rank: rk, pivots: p) = m.rref()
        if grid(got) != e or rk != piv.len or p != piv: bad = "op {k}: rref {g}"
        elif m.rank() != piv.len: bad = "op {k}: rank {g}"
    elif op == 5:
        let b = [F(rng.int(0, 6)) for _ in 0..r]
        let (e, piv) = rref_b([g[i] + [b[i]] for i in 0..r], c + 1)
        var want: ?(List<F>, List<List<F>>) = None
        if not piv.contains(c):
            var x = List.filled(c, F(0))
            for i in 0..piv.len: x[piv[i]] = e[i][c]
            var basis = List<List<F>>()
            for f in 0..c:
                if piv.contains(f): continue
                var v = List.filled(c, F(0))
                v[f] = F(1)
                for i in 0..piv.len: v[piv[i]] = F(0) - e[i][f]
                basis.push(v)
            want = Some((x, basis))
        if m.solve_space(b) != want: bad = "op {k}: solve_space {g} {b}"
        elif m.solve(b) != want.map(w => w.0): bad = "op {k}: solve {g} {b}"
        elif want != None and m.mul_vec(m.solve(b)!) != b: bad = "op {k}: solve does not solve {g} {b}"
    elif op == 6:
        let cp = sq.char_poly()
        var ok = cp.len == s + 1 and cp[s] == F(1)
        for t in 0..=s:
            let xt = [[(if i == j: F(t) else: F(0)) - q[i][j] for j in 0..s] for i in 0..s]
            var (v, pw) = (F(0), F(1))
            for co in cp:
                v += co * pw
                pw *= F(t)
            if v != det_b(xt): ok = false
        if not ok: bad = "op {k}: char_poly {q}"
    elif op == 7:
        let h = 2 * rng.int(0, 3)
        var sk = [List.filled(h, F(0)) for _ in 0..h]
        for i in 0..h:
            for j in i + 1..h:
                sk[i][j] = F(rng.int(0, 6))
                sk[j][i] = F(0) - sk[i][j]
        let pf = Mat<F>.of(sk).pfaffian()
        if pf != pf_b(sk) or pf * pf != det_b(sk): bad = "op {k}: pfaffian {sk}"
    elif op == 8:
        let mo = rng.int(1, 40)
        let a = [[rng.int(-20, 20) for _ in 0..s] for _ in 0..s]
        let d = det_int(a) % mo
        if Mat<Int>.of(a).det_mod(mo) != (if d < 0: d + mo else: d): bad = "op {k}: det_mod {a} {mo}"
    else:
        let a = [[if rng.int(0, 2) == 0: None else: Some(rng.int(-5, 9)) for _ in 0..c] for _ in 0..r]
        let b = [[if rng.int(0, 2) == 0: None else: Some(rng.int(-5, 9)) for _ in 0..s] for _ in 0..c]
        var x = Mat<?Int>.filled(r, c, None, over: Semiring(min, +))
        for i in 0..r:
            for j in 0..c: x.set(i, j, a[i][j])
        var y = Mat<?Int>.filled(c, s, None, over: Semiring(min, +))
        for i in 0..c:
            for j in 0..s: y.set(i, j, b[i][j])
        let z = x * y
        let mono = Mat<?Int>.product(c, over: Semiring(min, +))
        let ys = [[y.get(i, j) for j in 0..s] for i in 0..c]
        let none: ?Int = None
        var id = [List.filled(c, none) for _ in 0..c]
        for i in 0..c: id[i][i] = Some(0)
        if [[z.get(i, j) for j in 0..s] for i in 0..r] != mp(a, b, s): bad = "op {k}: min-plus {a} * {b}"
        elif [[mono.id.get(i, j) for j in 0..c] for i in 0..c] != id: bad = "op {k}: min-plus product({c}).id"
        elif s == c and [[(y ** 2).get(i, j) for j in 0..s] for i in 0..c] != mp(ys, ys, s): bad = "op {k}: min-plus {b} ** 2"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
