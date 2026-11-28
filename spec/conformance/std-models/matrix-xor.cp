// expect: ok, output "ok"
// input: "1 300\n"
// model: XorBasis
// model: PrefixXorBasis
// model: BitMat
// std.matrix's GF(2) structures against brute force: a XorBasis and a PrefixXorBasis against the
// whole span closed as a Set (values are XORs of a few random wide generators, so spans stay at
// most 2^7 elements while every bit position gets used), and BitMat against a List<List<Bool>>
// with a textbook elimination, the triple loop, and checking every answer back. Reads `seed n`;
// prints the first op that differs.
use std.matrix.{XorBasis, PrefixXorBasis, BitMat}
use std.rand.{Rng}

fn close(vs: List<Int>) -> Set<Int>:
    var s = Set<Int>()
    s.insert(0)
    for v in vs:
        if v in s: continue
        for x in [y for y in s]: s.insert(x ^ v)
    s

fn span_list(vs: List<Int>) -> List<Int> = [x for x in close(vs)].sorted()

fn draw(mut rng: Rng, gens: List<Int>) -> Int:
    var x = 0
    for g in gens:
        if rng.int(0, 2) == 0: x ^= g
    x

fn log2(n: Int) -> Int:
    var (k, m) = (0, 1)
    while m < n:
        m *= 2
        k += 1
    k

fn rank_b(g: List<List<Bool>>, cols: Int) -> Int:
    var a = g.clone()
    var r = 0
    for c in 0..cols:
        var p = r
        while p < a.len and not a[p][c]: p += 1
        if p == a.len: continue
        (a[r], a[p]) = (a[p], a[r])
        for i in 0..a.len:
            if i != r and a[i][c]: a[i] = [a[i][j] != a[r][j] for j in 0..cols]
        r += 1
    r

fn mv(g: List<List<Bool>>, x: List<Bool>) -> List<Bool>:
    var out = List<Bool>()
    for row in g:
        var v = false
        for j in 0..x.len:
            if row[j] and x[j]: v = not v
        out.push(v)
    out

fn to_bm(g: List<List<Bool>>, r: Int, c: Int) -> BitMat:
    var m = BitMat(r, c)
    for i in 0..r:
        for j in 0..c: m.set(i, j, g[i][j])
    m

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
let gens = [rng.int(1, 2 ** 62 - 1) for _ in 0..7] + [1 << rng.int(0, 61), rng.int(1, 15)]
var basis = XorBasis()
var added = List<Int>()
var pre = PrefixXorBasis()
var pushed = List<Int>()
for k in 0..n:
    let op = rng.int(0, 6)
    if op == 0:
        let x = if rng.int(0, 5) == 0: 0 else: draw(mut rng, [gens[rng.int(0, gens.len - 1)] for _ in 0..rng.int(1, 3)])
        let span = close(added)
        let got = basis.add(x)
        added.push(x)
        if got != (x not in span): bad = "op {k}: XorBasis.add({x})"
    elif op == 1:
        let span = span_list(added)
        let x = draw(mut rng, gens)
        let init = draw(mut rng, gens)
        let vs = basis.basis_vecs()
        let lo = min(x ^ s for s in span)
        let hi = max(init ^ s for s in span)
        let kk = rng.int(-1, span.len)
        let want_k: ?Int = if 0 <= kk < span.len: Some(span[kk]) else: None
        if basis.rank != log2(span.len) or 1 << basis.rank != span.len: bad = "op {k}: XorBasis.rank"
        elif basis.reduce(x) != lo or basis.min_xor(x) != lo or basis.min_xor() != 0: bad = "op {k}: XorBasis.reduce({x})"
        elif basis.contains(x) != (lo == 0): bad = "op {k}: XorBasis.contains({x})"
        elif basis.max_xor(init) != hi or basis.max_xor() != span.last!: bad = "op {k}: XorBasis.max_xor({init})"
        elif basis.kth(kk) != want_k: bad = "op {k}: XorBasis.kth({kk})"
        elif vs.len != basis.rank or span_list(vs) != span or any(vs[i] <= vs[i + 1] for i in 0..vs.len - 1): bad = "op {k}: XorBasis.basis_vecs {vs}"
        else:
            // fully reduced: no vector has another's highest bit set
            for i in 0..vs.len:
                var hb = 62
                while (vs[i] >> hb) & 1 == 0: hb -= 1
                for j in 0..vs.len:
                    if i != j and (vs[j] >> hb) & 1 == 1: bad = "op {k}: XorBasis.basis_vecs not reduced {vs}"
    elif op == 2:
        var other = XorBasis()
        var ov = List<Int>()
        for _ in 0..rng.int(0, 4):
            let x = draw(mut rng, gens)
            other.add(x)
            ov.push(x)
        let (sa, sb) = (close(added), close(ov))
        let inter = basis.intersect(other)
        let want_i = [x for x in sa if x in sb].sorted()
        let m = XorBasis.monoid()
        let joined = m.op(basis, other)
        let want_u = span_list(added + ov)
        if span_list(inter.basis_vecs()) != want_i or inter.rank != log2(want_i.len): bad = "op {k}: XorBasis.intersect"
        elif m.id.rank != 0 or span_list(joined.basis_vecs()) != want_u: bad = "op {k}: XorBasis.monoid"
        elif rng.int(0, 1) == 0:
            basis.merge(other)
            added = added + ov
            if span_list(basis.basis_vecs()) != want_u: bad = "op {k}: XorBasis.merge"
    elif op == 3:
        let x = if rng.int(0, 5) == 0: 0 else: draw(mut rng, gens) & (2 ** 62 - 1)
        pre.push(x)
        pushed.push(x)
    elif op == 4:
        let l = rng.int(0, pushed.len)
        let r = rng.int(l, pushed.len)
        let span = close([pushed[i] for i in l..r])
        let x = draw(mut rng, gens) & (2 ** 62 - 1)
        let init = draw(mut rng, gens)
        if pre.rank(l..r) != log2(span.len): bad = "op {k}: PrefixXorBasis.rank({l}..{r})"
        elif pre.contains(l..r, x) != (x in span): bad = "op {k}: PrefixXorBasis.contains({l}..{r}, {x})"
        elif pre.max_xor(l..r, init) != max(init ^ s for s in span): bad = "op {k}: PrefixXorBasis.max_xor({l}..{r}, {init})"
        elif pre.max_xor(l..r) != max(s for s in span): bad = "op {k}: PrefixXorBasis.max_xor({l}..{r})"
    else:
        let wide = rng.int(0, 7) == 0
        let (r, c) = (if wide: rng.int(62, 66) else: rng.int(0, 6), if wide: rng.int(62, 66) else: rng.int(0, 6))
        let dense = rng.int(1, 4)
        var g = [[rng.int(0, dense) == 0 for _ in 0..c] for _ in 0..r]
        if r >= 2 and rng.int(0, 1) == 0: g[rng.int(0, r - 1)] = g[rng.int(0, r - 1)].clone()
        let m = to_bm(g, r, c)
        let rk = rank_b(g, c)
        let b = [rng.int(0, 1) == 0 for _ in 0..r]
        let solvable = rank_b([g[i] + [b[i]] for i in 0..r], c + 1) == rk
        let sol = m.solve(b)
        let sp = m.solve_space(b)
        let hc = rng.int(0, 5)
        let h = [[rng.int(0, 1) == 0 for _ in 0..hc] for _ in 0..c]
        let prod = m.mul(to_bm(h, c, hc))
        if any(m.get(i, j) != g[i][j] for i in 0..r, j in 0..c) or m.rows != r or m.cols != c: bad = "op {k}: BitMat get/set"
        elif m.rank() != rk: bad = "op {k}: BitMat.rank {r}x{c}"
        elif (sol != None) != solvable or (sol != None and mv(g, sol!) != b): bad = "op {k}: BitMat.solve {r}x{c}"
        elif (sp != None) != solvable or (sp != None and (sp!.x0 != sol! or sp!.basis.len != c - rk or any(mv(g, v) != List.filled(r, false) for v in sp!.basis))): bad = "op {k}: BitMat.solve_space {r}x{c}"
        elif any(prod.get(i, j) != (count(g[i][t] and h[t][j] for t in 0..c) % 2 == 1) for i in 0..r, j in 0..hc): bad = "op {k}: BitMat.mul"
        else:
            let s = if wide: rng.int(62, 66) else: rng.int(0, 6)
            var q = [[rng.int(0, 1) == 0 for _ in 0..s] for _ in 0..s]
            let sq = to_bm(q, s, s)
            let full = rank_b(q, s) == s
            let inv = sq.inverse()
            let id = BitMat.identity(s)
            let back = if inv != None: sq.mul(inv!) else: id
            if sq.det() != full or (inv != None) != full: bad = "op {k}: BitMat.det/inverse {s}"
            elif any(id.get(i, j) != (i == j) for i in 0..s, j in 0..s): bad = "op {k}: BitMat.identity({s})"
            elif full and any(back.get(i, j) != (i == j) for i in 0..s, j in 0..s): bad = "op {k}: BitMat.inverse {s}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
