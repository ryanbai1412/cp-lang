// expect: ok, output "ok"
// input: "1 300\n"
// model: smawk
// model: monotone_minima
// model: dnc_layer
// model: knuth
// model: aliens
// model: monge_path
// model: monge_paths
// model: MongePath
// The Monge and convexity DP speedups against their quadratic or cubic loops, on seeded random
// small instances: squared gaps between sorted points (Monge), interval sums (Knuth), convex
// sequences (aliens). Reads `seed n`; prints the first op that differs.
// std_functions: 1
use std.rand.{Rng}
use std.dp.{MongePath, aliens, dnc_layer, knuth, monge_path, monge_paths, monotone_minima, smawk}

fn sorted_pts(mut rng: Rng, n: Int) -> List<Int>:
    var a: List<Int> = []
    var x = rng.int(0, 5)
    for _ in 0..n:
        a.push(x)
        x += rng.int(0, 4)
    a

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 5)
    if op == 0:
        let r = rng.int(0, 6)
        let c = rng.int(1, 6)
        let xs = sorted_pts(mut rng, r)
        let ys = sorted_pts(mut rng, c)
        let ar = [rng.int(-9, 9) for _ in 0..r]
        let bc = [rng.int(-9, 9) for _ in 0..c]
        let hole = [rng.int(0, 3) == 0 for _ in 0..r]
        let f = (i: Int, j: Int) => (xs[i] - ys[j]) * (xs[i] - ys[j]) + ar[i] + bc[j]
        let s = smawk(r, c, f)
        let mm = monotone_minima(r, c, (i, j) => if hole[i]: None else: Some(f(i, j)))
        if s.len != r or mm.len != r: bad = "op {k}: smawk/monotone_minima len"
        for i in 0..r:
            if bad != "": break
            var bj = 0
            for j in 1..c:
                if f(i, j) < f(i, bj): bj = j
            if s[i].val != f(i, bj) or s[i].at != bj: bad = "op {k}: smawk row {i} = ({s[i].val}, {s[i].at}) want ({f(i, bj)}, {bj})"
            elif hole[i]:
                if mm[i] != None: bad = "op {k}: monotone_minima row {i} want None"
            elif mm[i] != Some((val: f(i, bj), at: bj)): bad = "op {k}: monotone_minima row {i}"
    elif op == 1:
        let m = rng.int(0, 8)
        let ps = sorted_pts(mut rng, m)
        let prev = [if rng.int(0, 3) == 0: None else: Some(rng.int(0, 20)) for _ in 0..m]
        let cost = (j: Int, i: Int) => (ps[i] - ps[j]) * (ps[i] - ps[j])
        let got = dnc_layer(prev, cost)
        if got.len != m: bad = "op {k}: dnc_layer len"
        for i in 0..m:
            if bad != "": break
            var want: ?Int = None
            for j in 0..i:
                if let p = prev[j]:
                    let v = p + cost(j, i)
                    if want == None or v < want!: want = Some(v)
            if got[i] != want: bad = "op {k}: dnc_layer at {i}"
    elif op == 2:
        let m = rng.int(0, 7)
        let ps = sorted_pts(mut rng, m + 1)
        let cost = (i: Int, j: Int) => ps[j] - ps[i]
        let got = knuth(m, cost)
        var dp = [[0 for _ in 0..=m] for _ in 0..=m]
        for len in 2..=m:
            for i in 0..=m - len:
                let j = i + len
                var b = -1
                for t in i + 1..j:
                    let v = dp[i][t] + dp[t][j]
                    if b < 0 or v < b: b = v
                dp[i][j] = b + cost(i, j)
        if got != dp: bad = "op {k}: knuth {m}"
    elif op == 3:
        let kmax = rng.int(0, 8)
        var ds = [rng.int(-10, 10) for _ in 0..kmax]
        ds.sort()
        var fs = [rng.int(-5, 5)]
        for d in ds: fs.push(fs[fs.len - 1] + d)
        let kk = rng.int(0, kmax)
        let solve = (lam: Int) =>
            var bv = fs[0]
            var bc = 0
            for c in 1..=kmax:
                let v = fs[c] + lam * c
                if v < bv:
                    bv = v
                    bc = c
            (val: bv, count: bc)
        let got = aliens(kk, solve)
        if got != fs[kk]: bad = "op {k}: aliens k={kk} = {got} want {fs[kk]}"
    else:
        let m = rng.int(1, 7)
        let nodes = sorted_pts(mut rng, m)
        let c0 = rng.int(0, 12)
        let cost = (a: Int, b: Int) => (b - a) * (b - a) + c0
        // best[i][e]: least total to node i using exactly e edges
        var best: List<List<?Int>> = [[None for _ in 0..m] for _ in 0..m]
        best[0][0] = Some(0)
        for i in 1..m:
            for j in 0..i:
                for e in 0..m - 1:
                    if let v = best[j][e]:
                        let w = v + cost(nodes[j], nodes[i])
                        if best[i][e + 1] == None or w < best[i][e + 1]!: best[i][e + 1] = Some(w)
        var opt: List<Int> = []
        for i in 0..m:
            var b: ?Int = None
            for e in 0..m:
                if let v = best[i][e]:
                    if b == None or v < b!: b = Some(v)
            opt.push(b!)
        let p = monge_path(nodes, cost)
        if p.val != opt[m - 1] or p.count < 0 or p.count >= m or best[m - 1][p.count] != Some(p.val):
            bad = "op {k}: monge_path = ({p.val}, {p.count}) want {opt[m - 1]}"
        let ps = monge_paths(nodes, cost)
        if bad == "" and (ps.val != opt or ps.prev.len != m or ps.prev[0] != 0): bad = "op {k}: monge_paths val"
        for i in 1..m:
            if bad != "": break
            let j = ps.prev[i]
            if j < 0 or j >= i or opt[j] + cost(nodes[j], nodes[i]) != opt[i]: bad = "op {k}: monge_paths prev {i}"
        var mp = MongePath(nodes.clone(), cost)
        if bad == "" and (mp.len != 1 or mp.is_empty or mp.val(0) != 0): bad = "op {k}: MongePath fresh"
        for i in 1..m:
            if bad != "": break
            let r = mp.next()
            if r.val != opt[i] or r.prev < 0 or r.prev >= i or opt[r.prev] + cost(nodes[r.prev], nodes[i]) != opt[i]:
                bad = "op {k}: MongePath.next {i}"
            elif mp.len != i + 1 or mp.val(i) != opt[i] or mp.is_empty: bad = "op {k}: MongePath.len/val {i}"
        if bad == "" and mp.items.collect() != [(i, opt[i]) for i in 0..m]: bad = "op {k}: MongePath.items"
    if bad != "": break
print(if bad == "": "ok" else: bad)
