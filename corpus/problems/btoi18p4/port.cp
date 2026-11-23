// Source: BOI 2018 P4 (Alternating Current), https://dmoj.ca/problem/btoi18p4
// Problem: a circle of n <= 1e5 segments and m <= 1e5 wires, each covering a cyclic range a..b.
//   Direct each wire (0 or 1) so every segment is covered by wires of both directions; print
//   the m digits, or "impossible".
// Solution: a wire inside another takes its container's opposite direction, so only the k
//   maximal arcs, in circle order, matter (sort by (l, -r), sweep the doubled circle). k even
//   or 1: alternate them. k <= 3: try every rotation. Otherwise k is odd, so two neighbours b, c
//   share a direction: that works iff every other maximal arc is covered by its children and
//   neighbours, and b..c by the children of b and c plus the arcs either side; try each pair,
//   alternate from c and verify with a coverage sweep. O((n + m) log m).
// original: corpus/submissions/4656863.cpp
let n: 1..=1e5 = read()
let m: 1..=1e5 = read()
domain Wire = 0..m

var arcs: List<(Int, Int, Int)> = []
for i in 0..m:
    let l_in: 1..=n = read()
    let r_in: 1..=n = read()
    let l = l_in - 1
    var r = r_in - 1
    if l > r: r += n
    arcs.push((l, -r, i))
arcs.sort()

var al: [0..m]Int = 0
var ar: [0..m]Int = 0
var aid: [0..m]Int = 0
for i in 0..m:
    let t = arcs[i]
    al[i] = t.0
    var r = -t.1
    if r >= n: r -= n
    ar[i] = r
    aid[i] = t.2

var par: [0..m]Int = 0
var best_r = -1
var best_i = -1
for step in 0..2 * m:
    let ci = if step >= m: step - m else: step
    var l = al[ci]
    var r = ar[ci]
    if l > r: r += n
    if step >= m:
        l += n
        r += n
    if r <= best_r:
        par[ci] = best_i
    else:
        par[ci] = ci
        best_r = r
        best_i = ci

for i in 0..m:
    var u = i
    while par[u] != u: u = par[u]
    par[i] = u

var vals: List<Int> = []
var inds: [0..m]Int = 0
for i in 0..m:
    if par[i] == i:
        inds[i] = vals.len
        vals.push(i)

fn covers_all(segs: List<(Int, Int)>) -> Bool:
    var by_l: [0..n]List<Int> = []
    for idx in 0..segs.len:
        let s = segs[idx]
        by_l[s.0].push(s.1)
    var reach = -1
    for i in 0..n:
        for r in by_l[i]: reach max= r
        if reach < i: return false
    true

fn covers_arc(segs: List<(Int, Int)>, ql: Int, qr: Int) -> Bool:
    if segs.len == 0: return false
    var evts: List<(Int, Int)> = []
    for idx in 0..segs.len:
        let s = segs[idx]
        let l = s.0
        let r = s.1
        evts.push((l - n, 1))
        evts.push((r - n + 1, -1))
        evts.push((l, 1))
        evts.push((r + 1, -1))
        evts.push((l + n, 1))
        evts.push((r + n + 1, -1))
    evts.sort()
    var prev: ?Int = None
    var cur = 0
    for idx in 0..evts.len:
        let e = evts[idx]
        let pos = e.0
        if let p = prev:
            if pos != p and cur == 0:
                if (ql <= pos - 1 and pos - 1 <= qr) or (ql <= p and p <= qr):
                    return false
        cur += e.1
        prev = pos
    true

let k = vals.len
domain K = 0..k

fn try_color(distr: [K]Bool) -> Bool:
    var ans: [Wire]Int = 0
    var s0: List<(Int, Int)> = []
    var s1: List<(Int, Int)> = []
    for i in 0..m:
        let l = al[i]
        let r = ar[i]
        let ci = aid[i]
        let d = distr[K(inds[par[i]])]
        let bit = if par[i] == i: d else: not d
        ans[Wire(ci)] = Int(bit)
        if bit:
            if l > r:
                s1.push((l, n - 1))
                s1.push((0, r))
            else:
                s1.push((l, r))
        else:
            if l > r:
                s0.push((l, n - 1))
                s0.push((0, r))
            else:
                s0.push((l, r))
    if covers_all(s0) and covers_all(s1):
        var row: [Wire]Char = '0'
        for w in Wire:
            if ans[w] == 1: row[w] = '1'
        print(row)
        return true
    false

fn solve():
    if k % 2 == 0 or k == 1:
        var distr: [K]Bool = false
        for i in K: distr[i] = (i.idx % 2 == 1)
        if not try_color(distr): print("impossible")
        return
    if k <= 3:
        for start in K:
            var distr: [K]Bool = false
            var j = start
            var cur = true
            for _ in 0..k:
                distr[j] = cur
                cur = not cur
                j = K.wrap(j.ord + 1)
            if try_color(distr): return
        print("impossible")
        return

    var all_s: List<(Int, Int)> = []
    for i in 0..m:
        let l = al[i]
        let r = ar[i]
        if l > r:
            all_s.push((l, n - 1))
            all_s.push((0, r))
        else:
            all_s.push((l, r))
    if not covers_all(all_s):
        print("impossible")
        return

    var child: [K]List<(Int, Int)> = []
    for i in 0..m:
        let r = if al[i] > ar[i]: ar[i] + n else: ar[i]
        if par[i] != i:
            child[K(inds[par[i]])].push((al[i], r))
    var cl: [K]Int = 0
    var cr: [K]Int = 0
    for i in K:
        let idx = vals[i.idx]
        cl[i] = al[idx]
        cr[i] = if al[idx] > ar[idx]: ar[idx] + n else: ar[idx]

    var can: [K]Bool = false
    var ok_cnt = 0
    for i in K:
        let a = K.wrap(i.ord - 1)
        let b = K.wrap(i.ord + 1)
        var cur: List<(Int, Int)> = [s for s in child[i]]
        cur.push((cl[a], cr[a]))
        cur.push((cl[b], cr[b]))
        if covers_arc(cur, cl[i], cr[i]):
            can[i] = true
            ok_cnt += 1

    for i in K:
        let a = i
        let b = K.wrap(i.ord + 1)
        let c = K.wrap(i.ord + 2)
        let d = K.wrap(i.ord + 3)
        var ccnt = ok_cnt
        if can[b]: ccnt -= 1
        if can[c]: ccnt -= 1
        if ccnt != k - 2: continue
        var cur: List<(Int, Int)> = [s for s in child[b]]
        for s in child[c]: cur.push(s)
        cur.push((cl[a], cr[a]))
        cur.push((cl[d], cr[d]))
        if not covers_arc(cur, cl[b], cr[c]): continue
        var distr: [K]Bool = false
        var u = c
        var cb = true
        for _ in 0..k:
            distr[u] = cb
            cb = not cb
            u = K.wrap(u.ord + 1)
        if try_color(distr): return
    print("impossible")

solve()
