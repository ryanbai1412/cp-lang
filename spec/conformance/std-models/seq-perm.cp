// expect: ok, output "ok"
// input: "1 300\n"
// model: inverse
// model: perm_cycles
// model: perm_pow
// model: perm_rank
// model: perm_unrank
// model: perm_sign
// model: perm_order
// model: partial_inverse
// model: cycle
// model: combinations
// model: permutations
// model: subsequences
// model: SubseqStep
use std.rand.{Rng}
use std.seq.{perm_pow, perm_unrank}

domain V = 0..5

fn all_perms(n: Int) -> List<List<Int>>:
    if n == 0: return [[]]
    var out: List<List<Int>> = []
    for first in 0..n:
        for rest in all_perms(n - 1):
            out.push([first] + [if x >= first: x + 1 else: x for x in rest])
    out

fn apply(p: List<Int>, k: Int) -> List<Int>:
    var q = [i for i in 0..p.len]
    for _ in 0..k: q = [p[q[i]] for i in 0..p.len]
    q

fn gcd(a: Int, b: Int) -> Int = if b == 0: a else: gcd(b, a % b)

let perms = all_perms(5)
let seed = read<Int>()
let n = read<Int>()
var rng = Rng(seed)
var bad = ""
for op in 0..n:
    let r = rng.int(0, perms.len - 1)
    let pl = perms[r]
    let p: [V]V = [V] v => V(pl[v.ord])
    let k = rng.int(0, 7)
    var why = ""
    var cyc_want: List<List<Int>> = []
    var seen = [false for _ in 0..5]
    var inv_count = 0
    for i in 0..5:
        for j in i + 1..5:
            if pl[i] > pl[j]: inv_count += 1
    var order = 1
    for s in 0..5:
        if seen[s]: continue
        var c: List<Int> = []
        var u = s
        while not seen[u]:
            seen[u] = true
            c.push(u)
            u = pl[u]
        order = order / gcd(order, c.len) * c.len
        cyc_want.push(c)
    let from = rng.int(0, 4)
    let cyc_from = cyc_want.filter(c => c.contains(from)).first()!
    let rot = (0..cyc_from.len).filter(j => cyc_from[j] == from).first()!
    let walk = [cyc_from[(rot + j) % cyc_from.len] for j in 0..cyc_from.len]
    let inv = p.inverse()
    if [inv[V(pl[i])].ord for i in 0..5] != (0..5).collect(): why = "inverse"
    elif [[v.ord for v in c] for c in p.perm_cycles()] != cyc_want: why = "perm_cycles"
    elif [perm_pow(p, k)[v].ord for v in V] != apply(pl, k): why = "perm_pow {k}"
    elif p.perm_rank() != r: why = "perm_rank"
    elif [perm_unrank<V>(r)[v].ord for v in V] != pl: why = "perm_unrank {r}"
    elif p.perm_sign() != (if inv_count % 2 == 0: 1 else: -1): why = "perm_sign"
    elif p.perm_order() != order: why = "perm_order"
    elif [v.ord for v in p.cycle(from: V(from))] != walk: why = "cycle from {from}"
    if why == "":
        let hole = [rng.int(0, 2) == 0 for _ in 0..5]
        let part: [V]?V = [V] v => if hole[v.ord]: None else: Some(p[v])
        let pi = part.partial_inverse()
        let want = [(0..5).filter(i => not hole[i] and pl[i] == w).first() for w in 0..5]
        if [if let i = pi[V(w)]: Some(i.ord) else: None for w in 0..5] != want: why = "partial_inverse holes {[i for i in 0..5 if hole[i]]}"
    if why == "":
        let len = rng.int(0, 4)
        let xs = [rng.int(0, 3) for _ in 0..len]
        let m = rng.int(0, len + 1)
        let combos = [[xs[i] for i in 0..len if (mk >> i) & 1 == 1] for mk in 0..(1 << len) if (0..len).count(i => (mk >> i) & 1 == 1) == m]
        var cw: List<List<Int>> = []
        // lexicographic index order: compare index lists
        var idxs = [[i for i in 0..len if (mk >> i) & 1 == 1] for mk in 0..(1 << len) if (0..len).count(i => (mk >> i) & 1 == 1) == m]
        idxs.sort()
        cw = [[xs[i] for i in ix] for ix in idxs]
        let pk = rng.int(0, len)
        var pidx: List<List<Int>> = []
        for pp in all_perms(len):
            let pre = [pp[j] for j in 0..pk]
            if not pidx.any(q => q == pre): pidx.push(pre)
        pidx.sort()
        let pw = [[xs[i] for i in ix] for ix in pidx]
        var sidx = [[i for i in 0..len if (mk >> i) & 1 == 1] for mk in 0..(1 << len)]
        sidx.sort()
        let sw = [[xs[i] for i in ix] for ix in sidx]
        let cap = rng.int(0, len)
        let pruned_want = [[xs[i] for i in ix] for ix in sidx if ix.len <= cap]
        var pruned_got: List<List<Int>> = []
        for s in xs.subsequences():
            pruned_got.push(s.items)
            if s.items.len >= cap: s.prune()
        if combos.len != cw.len or xs.combinations(m).collect() != cw: why = "combinations {xs} {m}"
        elif xs.permutations(k: Some(pk)).collect() != pw: why = "permutations {xs} k {pk}"
        elif len == pk and xs.permutations().collect() != pw: why = "permutations {xs}"
        elif xs.subsequences().map(s => s.items).collect() != sw: why = "subsequences {xs}"
        elif pruned_got != pruned_want: why = "subsequences prune at {cap} {xs}"
    if why != "":
        bad = "op {op}: {why} of {pl}"
        break
print(if bad == "": "ok" else: bad)
