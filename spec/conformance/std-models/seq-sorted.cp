// expect: ok, output "ok"
// input: "1 300\n"
// model: merge_sorted
// model: sorted_union
// model: sorted_intersect
// model: sorted_diff
// model: kway_merge
// model: first_gt
// model: last_lt
// model: equal_range
// model: count_in
// model: span
// model: is_permutation
// model: is_subsequence
// model: lcs
// model: lis
// model: buckets
// model: compress
// model: Compressed
// model: with_domain
// model: pieces
// model: covering
use std.rand.{Rng}
use std.seq.{kway_merge, lcs}

fn rand_sorted(mut rng: Rng, n: Int) -> List<Int> = [rng.int(0, 6) for _ in 0..n].sorted()

domain W = 0..9

fn wcut(i: Int) -> Cut<W> = if i < 9: W(i).l else: W(8).r

fn occ(a: List<Int>, x: Int) -> Int = a.count(v => v == x)

fn subseq_of(s: List<Int>, mask: Int) -> List<Int> = [s[i] for i in 0..s.len if (mask >> i) & 1 == 1]

let seed = read<Int>()
let n = read<Int>()
var rng = Rng(seed)
var bad = ""
for op in 0..n:
    let a = rand_sorted(mut rng, rng.int(0, 6))
    let b = rand_sorted(mut rng, rng.int(0, 6))
    let t = rng.int(-1, 7)
    var why = ""
    let both = [x for x in 0..=6, _ in 0..occ(a, x) + occ(b, x)]
    let uni = [x for x in 0..=6 if occ(a, x) + occ(b, x) > 0]
    let inter = [x for x in 0..=6, _ in 0..min(occ(a, x), occ(b, x))]
    let diff = [x for x in 0..=6, _ in 0..max(occ(a, x) - occ(b, x), 0)]
    let c = rand_sorted(mut rng, rng.int(0, 4))
    let three = [x for x in 0..=6, _ in 0..occ(a, x) + occ(b, x) + occ(c, x)]
    let gt = (0..a.len).filter(i => a[i] > t).first()
    let lt = (0..a.len).filter(i => a[i] < t).last()
    let eq = [i for i in 0..a.len if a[i] == t]
    let lo = rng.int(-1, 7)
    let hi = rng.int(-1, 7)
    let inr = a.count(v => lo <= v < hi)
    let lo2 = rng.int(0, 5)
    let len2 = rng.int(0, 3)
    if a.merge_sorted(b) != both: why = "merge_sorted {a} {b}"
    elif a.sorted_union(b) != uni: why = "sorted_union {a} {b}"
    elif a.sorted_intersect(b) != inter: why = "sorted_intersect {a} {b}"
    elif a.sorted_diff(b) != diff: why = "sorted_diff {a} {b}"
    elif kway_merge([a, b, c]) != three: why = "kway_merge {a} {b} {c}"
    elif a.first_gt(t) != gt: why = "first_gt {a} {t}"
    elif a.last_lt(t) != lt: why = "last_lt {a} {t}"
    elif a.equal_range(t).len != eq.len or (eq.len > 0 and a.equal_range(t).collect() != eq): why = "equal_range {a} {t}"
    elif a.count_in(lo..hi) != inr: why = "count_in {a} {lo}..{hi}"
    elif [w.ord for w in span(wcut(lo2), wcut(lo2 + len2))] != [lo2 + j for j in 0..len2]: why = "span {lo2} {len2}"
    if why == "":
        var u = a + b
        rng.shuffle(mut u)
        let same = (a + b).sorted() == u.sorted()
        var w = [rng.int(0, 3) for _ in 0..rng.int(0, 6)]
        let v = [rng.int(0, 3) for _ in 0..w.len + max(rng.int(-1, 1), -w.len)]
        let perm_b = w.sorted() == v.sorted()
        let q = [rng.int(0, 2) for _ in 0..rng.int(0, 4)]
        let p = [rng.int(0, 2) for _ in 0..rng.int(0, 6)]
        let sub = (0..(1 << p.len)).any(mk => subseq_of(p, mk) == q)
        var best_common = 0
        for m1 in 0..(1 << q.len):
            for m2 in 0..(1 << p.len):
                let s1 = subseq_of(q, m1)
                if s1.len > best_common and s1 == subseq_of(p, m2): best_common = s1.len
        let l = lcs(q, p)
        let l_ok = l.len == best_common and (0..l.len).all(k => q[l[k].0] == p[l[k].1] and (k == 0 or (l[k - 1].0 < l[k].0 and l[k - 1].1 < l[k].1)))
        let strict = rng.int(0, 1) == 1
        var best_inc = 0
        for mk in 0..(1 << p.len):
            let s1 = subseq_of(p, mk)
            if s1.len > best_inc and (1..s1.len).all(k => if strict: s1[k - 1] < s1[k] else: s1[k - 1] <= s1[k]): best_inc = s1.len
        let li = p.lis(strict:)
        let li_ok = li.len == best_inc and (1..li.len).all(k => li[k - 1] < li[k] and (if strict: p[li[k - 1]] < p[li[k]] else: p[li[k - 1]] <= p[li[k]]))
        if (a + b).is_permutation(u) != same or a.is_permutation(u) != (b.len == 0): why = "is_permutation {a} {u}"
        elif w.is_permutation(v) != perm_b: why = "is_permutation {w} {v}"
        elif q.is_subsequence(p) != sub: why = "is_subsequence {q} {p}"
        elif not l_ok: why = "lcs {q} {p}: {l}"
        elif not li_ok: why = "lis {p} strict {yes(strict)}: {li}"
    if why == "":
        domain K = 0..4
        let raw = [rng.int(0, 3) for _ in 0..rng.int(0, 6)]
        let (domain I, xs) = raw.with_domain()
        let ks: [I]K = [I] i => K(xs[i])
        let bk = ks.buckets()
        let bk_ok = (0..4).all(k => [i.ord for i in bk[K(k)]] == [j for j in 0..raw.len if raw[j] == k])
        let dom_ok = I.count == raw.len and (0..raw.len).all(j => xs[I(j)] == raw[j])
        let (domain C, cs) = compress(raw)
        let vals = [x for x in 0..=3 if occ(raw, x) > 0]
        let probe = rng.int(-1, 4)
        let at = (j: ?C) => if let k = j: k.ord else: -1
        let fge = (0..vals.len).filter(j => vals[j] >= probe).first() ?? -1
        let fgt = (0..vals.len).filter(j => vals[j] > probe).first() ?? -1
        let lle = (0..vals.len).filter(j => vals[j] <= probe).last() ?? -1
        let llt = (0..vals.len).filter(j => vals[j] < probe).last() ?? -1
        let has = occ(vals, probe) > 0
        if not dom_ok: why = "with_domain {raw}"
        elif not bk_ok: why = "buckets {raw}"
        elif cs.len != vals.len or cs.is_empty != (vals.len == 0): why = "compress len {raw}"
        elif [cs.get(i) for i in C] != vals or cs.slice(C.all) != vals: why = "compress values {raw}"
        elif [v for (_, v) in cs.items] != vals or [i.ord for (i, _) in cs.items] != (0..vals.len).collect(): why = "compress items {raw}"
        elif cs.contains(probe) != has or at(cs.try_index_of(probe)) != (if has: vals.count(v => v < probe) else: -1): why = "compress lookup {raw} {probe}"
        elif has and cs.index_of(probe).ord != vals.count(v => v < probe): why = "compress index_of {raw} {probe}"
        elif at(cs.first_ge(probe)) != fge or at(cs.first_gt(probe)) != fgt: why = "compress first_ge/gt {raw} {probe}"
        elif at(cs.last_le(probe)) != lle or at(cs.last_lt(probe)) != llt: why = "compress last_le/lt {raw} {probe}"
    if why == "":
        let ln = rng.int(0, 8)
        let cuts = [rng.int(0, ln) for _ in 0..rng.int(0, 4)]
        let (domain P, piece) = (0..ln).pieces(cuts:)
        let ends = [x for x in 0..=ln if x == 0 or x == ln or occ(cuts, x) > 0]
        let want = if ln == 0: [] else: [(ends[j], ends[j + 1]) for j in 0..ends.len - 1]
        let got = [(piece[p].l, piece[p].r) for p in P]
        if got != want: why = "pieces 0..{ln} cuts {cuts}: {got}"
        elif ln > 0:
            let x = rng.int(0, ends.len - 1)
            let y = rng.int(x, ends.len - 1)
            let cov = piece.covering(ends[x]..ends[y])
            if [p.ord for p in cov] != (x..y).collect(): why = "covering {ends[x]}..{ends[y]} of {want}"
    if why != "":
        bad = "op {op}: {why}"
        break
print(if bad == "": "ok" else: bad)
