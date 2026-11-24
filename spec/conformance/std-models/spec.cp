// expect: ok, output "ok"
// input: "1 300\n"
// model: decrease
// model: increase
// model: convexity_break
// model: concavity_break
// model: unimodality_break
// model: monge_break
// model: argmins
// model: dnc_break
// model: knuth_break
// model: differences
// model: newton
// model: poly_degree
// model: poly_at
// model: berlekamp_massey
// model: recurrence_at
// model: guess_at
use std.rand.{Rng}
use std.mod.{Mod}
use std.spec.{argmins, berlekamp_massey, concavity_break, convexity_break, decrease, differences, dnc_break, guess_at, increase, knuth_break, monge_break, newton, poly_at, poly_degree, recurrence_at, unimodality_break}

type Z = Mod<7>

fn first(hits: List<Int>) -> ?Int = if hits.is_empty: None else: Some(hits.min()!)

fn choose(n: Int, k: Int) -> Int:
    var c = 1
    for i in 0..k: c = c * (n - i) / (i + 1)
    c

// Rising strictly, then a plateau, then falling strictly: some peak run `p..=q` splits it.
fn unimodal(ys: List<Int>) -> Bool:
    for p in 0..ys.len:
        for q in p..ys.len:
            if (0..p).all(i => ys[i] < ys[i + 1]) and (p..q).all(i => ys[i] == ys[i + 1]) and (q..ys.len - 1).all(i => ys[i] > ys[i + 1]): return true
    ys.is_empty

// The next term after `seq[..i]` under recurrence `c`.
fn step(c: List<Z>, seq: List<Z>, i: Int) -> Z:
    var t = Z(0)
    for j in 0..c.len: t += c[j] * seq[i - 1 - j]
    t

fn leftmost_min(row: List<Int>) -> Int:
    var b = 0
    for j in 0..row.len:
        if row[j] < row[b]: b = j
    b

fn satisfies(xs: List<Z>, c: List<Z>) -> Bool =
    (c.len..xs.len).all(i => xs[i] == step(c, xs, i))

fn code(l: Int, m: Int) -> List<Z>:
    var c: List<Z> = []
    var x = m
    for _ in 0..l:
        c.push(Z(x % 7))
        x /= 7
    c

// Every recurrence of the shortest length up to 3 that `xs` satisfies, by trying them all.
fn shortest(xs: List<Z>) -> List<List<Z>>:
    for l in 0..=3:
        let hits = [code(l, m) for m in 0..7 ** l if satisfies(xs, code(l, m))]
        if not hits.is_empty: return hits
    []

let seed = read<Int>()
let n = read<Int>()
var rng = Rng(seed)
var bad = ""
for k in 0..n:
    if bad != "": break
    let op = rng.int(0, 6)
    let m = rng.int(0, 7)
    let xs = [rng.int(-3, 3) for _ in 0..m]
    if op == 0:
        let dn = first([i for i in 1..m if xs[i] < xs[i - 1]])
        let up = first([i for i in 1..m if xs[i] > xs[i - 1]])
        if decrease(xs) != dn or (dn == None) != (xs == xs.sorted()): bad = "op {k}: decrease({xs}) = {decrease(xs) ?? -1}"
        elif increase(xs) != up: bad = "op {k}: increase({xs}) = {increase(xs) ?? -1}"
        let d2 = differences(differences(xs))
        let vex = first([j + 1 for j in 0..d2.len if d2[j] < 0])
        let cav = first([j + 1 for j in 0..d2.len if d2[j] > 0])
        if bad == "" and convexity_break(xs) != vex: bad = "op {k}: convexity_break({xs}) = {convexity_break(xs) ?? -1}"
        elif bad == "" and concavity_break(xs) != cav: bad = "op {k}: concavity_break({xs}) = {concavity_break(xs) ?? -1}"
        let uni = first([i for i in 0..m if not unimodal(xs[0..i + 1].collect())])
        if bad == "" and unimodality_break(xs) != uni: bad = "op {k}: unimodality_break({xs}) = {unimodality_break(xs) ?? -1} want {uni ?? -1}"
    elif op == 1:
        let rows = rng.int(0, 4)
        let cols = rng.int(1, 4)
        let cs = [rng.int(-2, 2) for _ in 0..cols]
        let rs = [rng.int(-2, 2) for _ in 0..rows]
        let noisy = rng.chance(0.5)
        let t = [[(i - j) * (i - j) * 2 + rs[i] + cs[j] + (if noisy: rng.int(0, 2) else: 0) for j in 0..cols] for i in 0..rows]
        let cost = (i: Int, j: Int) => t[i][j]
        var want: ?(Int, Int) = None
        for i in (0..rows - 1).rev:
            for j in (0..cols - 1).rev:
                if t[i][j] + t[i + 1][j + 1] > t[i][j + 1] + t[i + 1][j]: want = Some((i, j))
        let full = (0..rows).all(a => (a + 1..rows).all(b => (0..cols).all(c => (c + 1..cols).all(d => t[a][c] + t[b][d] <= t[a][d] + t[b][c]))))
        let got = monge_break(cost, rows, cols)
        if got != want or (got == None) != full: bad = "op {k}: monge_break({t}) = {got ?? (-1, -1)}"
        let am = [leftmost_min(t[i]) for i in 0..rows]
        if bad == "" and argmins(cost, rows, cols) != am: bad = "op {k}: argmins({t}) = {argmins(cost, rows, cols)} want {am}"
    elif op == 2:
        let sz = rng.int(0, 6)
        let noisy = rng.chance(0.5)
        let t = [[(i - j) * (i - j) + (if noisy: rng.int(0, 3) else: 0) for i in 0..sz] for j in 0..sz]
        var asked_bad = false
        let cost = (j: Int, i: Int) =>
            if j >= i: asked_bad = true
            t[j][i]
        var want: ?(Int, Int) = None
        for a in (0..sz).rev:
            for c in (a + 2..sz - 1).rev:
                if t[a][c] + t[a + 1][c + 1] > t[a][c + 1] + t[a + 1][c]: want = Some((a, c))
        let got = dnc_break(cost, sz)
        if got != want or asked_bad: bad = "op {k}: dnc_break({t}) = {got ?? (-1, -1)} want {want ?? (-1, -1)}"
        let o = [[rng.int(0, sz) for _ in 0..=sz] for _ in 0..=sz]
        var asked_short = false
        let opt = (i: Int, j: Int) =>
            if j - i < 2: asked_short = true
            if noisy: o[i][j] else: (i + j) / 2
        var kw: ?(Int, Int) = None
        for len in (3..=sz).rev:
            for i in (0..=sz - len).rev:
                let j = i + len
                let x = if noisy: o[i][j] else: (i + j) / 2
                let lo = if noisy: o[i][j - 1] else: (i + j - 1) / 2
                let hi = if noisy: o[i + 1][j] else: (i + 1 + j) / 2
                if x < lo or x > hi: kw = Some((i, j))
        let kg = knuth_break(opt, sz)
        if bad == "" and (kg != kw or asked_short): bad = "op {k}: knuth_break({o}, {sz}) = {kg ?? (-1, -1)} want {kw ?? (-1, -1)}"
    elif op == 3:
        let dg = rng.int(0, 3)
        let co = [rng.int(-3, 3) for _ in 0..=dg]
        let ys = if rng.chance(0.3): xs else: [(0..=dg).map(e => co[e] * x ** e).sum() for x in 0..m]
        let df = [ys[i + 1] - ys[i] for i in 0..ys.len - 1]
        if differences(ys) != df: bad = "op {k}: differences({ys}) = {differences(ys)}"
        let nw = newton(ys)
        if bad == "" and (nw.len != ys.len or (0..ys.len).any(x => (0..nw.len).map(j => nw[j] * choose(x, j)).sum() != ys[x])): bad = "op {k}: newton({ys}) = {nw}"
        var deg: ?Int = None
        var row = df
        var d = 0
        while deg == None and not row.is_empty:
            if row.all(x => x == 0): deg = Some(d)
            row = [row[i + 1] - row[i] for i in 0..row.len - 1]
            d += 1
        if bad == "" and poly_degree(ys) != deg: bad = "op {k}: poly_degree({ys}) = {poly_degree(ys) ?? -1} want {deg ?? -1}"
        let at = rng.int(-6, 12)
        if bad == "" and ys != xs and m > dg and poly_at(ys, at) != (0..=dg).map(e => co[e] * at ** e).sum(): bad = "op {k}: poly_at({ys}, {at}) = {poly_at(ys, at)}"
        if bad == "" and m == 0 and poly_at(ys, at) != 0: bad = "op {k}: poly_at([], {at}) = {poly_at(ys, at)}"
    elif op == 4:
        let l = rng.int(0, 3)
        let c = [Z(rng.int(0, 6)) for _ in 0..l]
        let init = [Z(rng.int(0, 6)) for _ in 0..l]
        var seq = init.clone()
        while seq.len < 40: seq.push(step(c, seq, seq.len))
        if l == 0: seq = List.filled(40, Z(0))
        let at = rng.int(0, 39)
        let got = recurrence_at(c, init, at)
        if got != seq[at]: bad = "op {k}: recurrence_at({c}, {init}, {at}) = {got} want {seq[at]}"
    else:
        let zs = if rng.chance(0.5): [Z(rng.int(0, 6)) for _ in 0..m] else: [Z(x) for x in xs]
        let bm = berlekamp_massey(zs)
        let all = shortest(zs)
        if not satisfies(zs, bm): bad = "op {k}: berlekamp_massey({zs}) = {bm}, which {zs} does not satisfy"
        elif not all.is_empty and bm.len != all[0].len: bad = "op {k}: berlekamp_massey({zs}) = {bm}, shortest {all[0]}"
        elif all.is_empty and bm.len <= 3: bad = "op {k}: berlekamp_massey({zs}) = {bm}, but none that short exists"
        elif all.len == 1 and bm != all[0]: bad = "op {k}: berlekamp_massey({zs}) = {bm} want {all[0]}"
        let at = rng.int(0, 12)
        var want: ?Z = None
        if at < zs.len: want = Some(zs[at])
        elif not all.is_empty and zs.len > 2 * all[0].len:
            var seq = zs.clone()
            let c = all[0]
            while seq.len <= at: seq.push(step(c, seq, seq.len))
            want = Some(seq[at])
        let got = guess_at(zs, at)
        if bad == "" and got != want: bad = "op {k}: guess_at({zs}, {at}) = {got ?? Z(0)} want {want ?? Z(0)}"
print(if bad == "": "ok" else: bad)
