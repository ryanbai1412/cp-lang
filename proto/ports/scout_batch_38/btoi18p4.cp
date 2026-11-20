// Baltic OI '18 P4 (Alternating Current): 2-color m circular arcs on a ring
// of n segments so each color covers the entire ring, or print "impossible".
// original: corpus/submissions/4656863.cpp
let (n, m): (1..=1e5, 1..=1e5) = read()
domain Wire = 0..m
type Arc = (l: Int, r: Int, id: Wire)
var arcs: [domain I = 0..m]Arc = [I] i =>
    let (l, r): (1..=n, 1..=n) = read()
    let rr = if l - 1 > r - 1: r - 1 + n else: r - 1
    (l - 1, rr, Wire.from_idx(i.to_idx)!)
sort(mut arcs, by: a => (a.l, -a.r))
for i in I:
    if arcs[i].r >= n: arcs[i].r -= n

// Eliminate arcs strictly contained in another arc via a 2m circular sweep.
var par: [I]I = [I] i => i
var best_r = -1
var best_i: ?I = none
for step in 0..2 * m:
    let i = I.from_idx(step % m)!
    var (l, r) = (arcs[i].l, arcs[i].r)
    if l > r: r += n
    if step >= m: (l, r) = (l + n, r + n)
    if r <= best_r: par[i] = best_i!
    else:
        par[i] = i
        (best_r, best_i) = (r, i)
for i in I:
    var u = i
    while par[u] != u: u = par[u]
    par[i] = u

var max_list: List<I> = []
for i in I if par[i] == i: max_list.push(i)
let (domain K, max_ids) = max_list.freeze()
var rank_of: [I]?K = none
for (k, id) in max_ids.items: rank_of[id] = k

fn covers_all(segs: List<(Int, Int)>) -> Bool:
    var by_l: [0..n]List<Int> = []
    for (l, r) in segs: by_l[l].push(r)
    var reach = -1
    for i in 0..n:
        for r in by_l[i]: reach max= r
        if reach < i: return false
    true

fn split_arc(l: Int, r: Int) -> List<(Int, Int)>:
    if l > r: [(l, n - 1), (0, r)] else: [(l, r)]

fn try_color(distr: [K]Bool) -> ?[Wire]Bool:
    var ans: [Wire]Bool = false
    var segs: [Bool]List<(Int, Int)> = []
    for i in I:
        let k = rank_of[par[i]]!
        let c = if par[i] == i: distr[k] else: not distr[k]
        ans[arcs[i].id] = c
        for s in split_arc(arcs[i].l, arcs[i].r): segs[c].push(s)
    if covers_all(segs[false]) and covers_all(segs[true]): ans else: none

fn covers_arc(segs: List<(Int, Int)>, ql: Int, qr: Int) -> Bool:
    if segs.is_empty: return false
    var evts: List<(Int, Int)> = []
    for (l, r) in segs:
        for shift in [-n, 0, n]:
            evts.push((l + shift, 1))
            evts.push((r + shift + 1, -1))
    sort(mut evts)
    var prev: ?Int = none
    var cur = 0
    for (pos, delta) in evts:
        if let p = prev and pos != p and cur == 0:
            if (ql <= pos - 1 and pos - 1 <= qr) or (ql <= p and p <= qr):
                return false
        cur += delta
        prev = pos
    true

fn solve() -> ?[Wire]Bool:
    if K.K % 2 == 0 or K.K == 1:
        return try_color([K] i => i.to_idx % 2 == 1)
    if K.K == 3:
        for start in K:
            var distr: [K]Bool = false
            var j = start
            var c = true
            for _ in 0..K.K:
                distr[j] = c
                c = not c
                j = j.cyc(1)
            if let ok = try_color(distr): return ok
        return none

    var all_segs: List<(Int, Int)> = []
    for i in I:
        for s in split_arc(arcs[i].l, arcs[i].r): all_segs.push(s)
    if not covers_all(all_segs): return none

    var child: [K]List<(Int, Int)> = []
    for i in I if par[i] != i:
        let r = if arcs[i].l > arcs[i].r: arcs[i].r + n else: arcs[i].r
        child[rank_of[par[i]]!].push((arcs[i].l, r))
    let carr: [K](Int, Int) = [K] k => (
        arcs[max_ids[k]].l,
        if arcs[max_ids[k]].l > arcs[max_ids[k]].r: arcs[max_ids[k]].r + n else: arcs[max_ids[k]].r,
    )

    var can: [K]Bool = false
    for i in K:
        let cur = child[i] + [carr[i.cyc(-1)], carr[i.cyc(1)]]
        can[i] = covers_arc(cur, carr[i].0, carr[i].1)
    let ok_cnt = count(can)

    for i in K:
        let (a, b, c, d) = (i, i.cyc(1), i.cyc(2), i.cyc(3))
        if ok_cnt - (if can[b]: 1 else: 0) - (if can[c]: 1 else: 0) != K.K - 2:
            continue
        let cur = child[b] + child[c] + [carr[a], carr[d]]
        if not covers_arc(cur, carr[b].0, carr[c].1): continue
        var distr: [K]Bool = false
        var u = c
        var bit = true
        for _ in 0..K.K:
            distr[u] = bit
            bit = not bit
            u = u.cyc(1)
        if let ok = try_color(distr): return ok
    none

if let ans = solve():
    print("".join(if ans[w]: "1" else: "0" for w in Wire))
else:
    print("impossible")
