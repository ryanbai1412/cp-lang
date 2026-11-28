// expect: ok, output "ok"
// input: "1 300\n"
// model: sparse_det
// model: sparse_solve
// model: simplex
// std.matrix's sparse Wiedemann solvers and the simplex method against brute force: sparse_det and
// sparse_solve against Gaussian elimination on the dense table the entries add up to (prime 7 too,
// where singular matrices and unlucky random projections are common), and simplex on small integer
// LPs against every vertex: each choice of n tight constraints among the rows, x >= 0 and a box
// x <= M, solved exactly by Cramer's rule. The optimum is unbounded iff it grows with M, and
// infeasible iff no vertex is feasible. Reads `seed n`; prints the first op that differs.
use std.matrix.{sparse_det, sparse_solve, simplex, Mat}
use std.mod.{Mod}
use std.rand.{Rng}

type Z = Mod<998_244_353>
type S = Mod<7>

fn elim<P>(g: List<List<Mod<P>>>, b: List<Mod<P>>) -> (Mod<P>, ?List<Mod<P>>):
    let n = g.len
    var a = [g[i] + [b[i]] for i in 0..n]
    var d = Mod<P>(1)
    for c in 0..n:
        var p = c
        while p < n and a[p][c] == Mod<P>(0): p += 1
        if p == n: return (Mod<P>(0), None)
        if p != c:
            (a[c], a[p]) = (a[p], a[c])
            d = Mod<P>(0) - d
        d *= a[c][c]
        let f = a[c][c].inv()
        a[c] = [x * f for x in a[c]]
        for i in 0..n:
            if i != c and a[i][c] != Mod<P>(0):
                let t = a[i][c]
                a[i] = [a[i][j] - t * a[c][j] for j in 0..=n]
    (d, Some([a[i][n] for i in 0..n]))

fn dense<P>(n: Int, es: List<(Int, Int, Mod<P>)>) -> List<List<Mod<P>>>:
    var g = [List.filled(n, Mod<P>(0)) for _ in 0..n]
    for (r, c, v) in es: g[r][c] += v
    g

fn det_int(a: List<List<Int>>) -> Int:
    if a.is_empty: return 1
    var s = 0
    for j in 0..a.len:
        let sub = [[a[i][t] for t in 0..a.len if t != j] for i in 1..a.len]
        let t = a[0][j] * det_int(sub)
        s = if j % 2 == 0: s + t else: s - t
    s

// The best c . x over vertices of {rows . x <= rhs, x >= 0, x <= big} as a fraction (num, den),
// or None when no vertex is feasible.
fn best(rows: List<List<Int>>, rhs: List<Int>, c: List<Int>, big: Int) -> ?(Int, Int):
    let n = c.len
    var cw = rows.clone()
    var ch = rhs.clone()
    for j in 0..n:
        cw.push([if t == j: -1 else: 0 for t in 0..n])
        ch.push(0)
        cw.push([if t == j: 1 else: 0 for t in 0..n])
        ch.push(big)
    var out: ?(Int, Int) = None
    for mask in 0..1 << cw.len:
        var pick = [i for i in 0..cw.len if (mask >> i) & 1 == 1]
        if pick.len != n: continue
        let a = [cw[i] for i in pick]
        var den = det_int(a)
        if den == 0: continue
        var num = [det_int([[if t == j: ch[pick[i]] else: a[i][t] for t in 0..n] for i in 0..n]) for j in 0..n]
        if den < 0:
            den = -den
            num = [-v for v in num]
        if any(sum(cw[i][t] * num[t] for t in 0..n) > ch[i] * den for i in 0..cw.len): continue
        let v = sum(c[t] * num[t] for t in 0..n)
        if out == None or v * out!.1 > out!.0 * den: out = Some((v, den))
    out

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 2)
    if op <= 1:
        let small = rng.int(0, 1) == 0
        let m = rng.int(0, 7)
        let cnt = rng.int(0, m * m + 3)
        let es = if m == 0: List<(Int, Int, Int)>() else: [(rng.int(0, m - 1), rng.int(0, m - 1), rng.int(0, 1_000_000)) for _ in 0..cnt]
        // a diagonal makes nonsingular matrices common
        let diag = if m > 0 and rng.int(0, 1) == 0: [(i, i, rng.int(1, 6)) for i in 0..m] else: List<(Int, Int, Int)>()
        let all = es + diag
        let bv = [rng.int(0, 1_000_000) for _ in 0..m]
        if small:
            let e = [(r, c, S(v)) for (r, c, v) in all]
            let b = [S(v) for v in bv]
            let (d, x) = elim(dense(m, e), b)
            if sparse_det(m, e) != d: bad = "op {k}: sparse_det mod 7 {m} {all}"
            elif sparse_solve(m, e, b) != x: bad = "op {k}: sparse_solve mod 7 {m} {all} {bv}"
        else:
            let e = [(r, c, Z(v)) for (r, c, v) in all]
            let b = [Z(v) for v in bv]
            let (d, x) = elim(dense(m, e), b)
            if sparse_det(m, e) != d: bad = "op {k}: sparse_det {m} {all}"
            elif sparse_solve(m, e, b) != x: bad = "op {k}: sparse_solve {m} {all} {bv}"
    else:
        let nv = rng.int(0, 3)
        let nr = rng.int(0, if nv == 3: 3 else: 4)
        let rows = [[rng.int(-4, 5) for _ in 0..nv] for _ in 0..nr]
        let rhs = [rng.int(-3, 10) for _ in 0..nr]
        let c = [rng.int(-3, 5) for _ in 0..nv]
        let lo = best(rows, rhs, c, 10_000)
        let hi = best(rows, rhs, c, 100_000)
        let want: ?F64 = if lo == None or lo!.0 * hi!.1 != hi!.0 * lo!.1: None else: Some(F64(lo!.0) / F64(lo!.1))
        let a = Mat<F64>.from_flat(nr, nv, [F64(rows[i][j]) for i in 0..nr, j in 0..nv])
        let got = simplex(a, [F64(v) for v in rhs], [F64(v) for v in c])
        if (got == None) != (want == None): bad = "op {k}: simplex {rows} {rhs} {c} feasible+bounded is {yes(want != None)}"
        elif got != None:
            let (val: val, x: x) = got!
            if abs(val - want!) > 1e-6: bad = "op {k}: simplex {rows} {rhs} {c} val {val}"
            elif x.len != nv or any(v < -1e-6 for v in x): bad = "op {k}: simplex {rows} {rhs} {c} x >= 0"
            elif any(sum(F64(rows[i][j]) * x[j] for j in 0..nv) > F64(rhs[i]) + 1e-6 for i in 0..nr): bad = "op {k}: simplex {rows} {rhs} {c} a x <= b"
            elif abs(sum(F64(c[j]) * x[j] for j in 0..nv) - val) > 1e-6: bad = "op {k}: simplex {rows} {rhs} {c} c . x"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
