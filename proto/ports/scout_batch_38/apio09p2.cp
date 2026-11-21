// APIO '09 P2 (Siruseri Convention Centre): find the maximum number of
// pairwise-disjoint closed intervals and the lexicographically smallest
// list of 1-based indices achieving it.
// original: corpus/submissions/5536701.cpp
let n: 1..=2e5 = read()
domain Orig = 1..=n
let raw: [Orig](l: Int, r: Int) = read()

type Seg = (l: Int, r: Int, id: Orig)
var segs: [domain I = 0..n]Seg = [I] i =>
    let orig = Orig.from_idx(i.to_idx)!
    (raw[orig].l - 1, raw[orig].r, orig)
segs.sort(by: s => (s.l, s.r))

var pos: [Orig]I = I.first
for (i, s) in segs.items: pos[s.id] = i

let st = SparseTable<I, (Int, I)>(min, [I] i => (segs[i].r, i))
let starts: [I]Int = [I] i => segs[i].l

// FRICTION 38.3: SparseTable.fold on an empty range with no-identity `min`
// needs `?T` or a manual empty guard.
fn min_r_in(lo: I, hi: I) -> ?(Int, I):
    if lo > hi: return none
    st.fold(lo..=hi)

fn next_from(x: Int, lo: I, hi: I) -> ?I:
    let start = max(starts.first_ge(x)?, lo)
    let (_, idx) = min_r_in(start, hi)?
    idx

// FRICTION 38.6: functional-graph binary lifting has no stdlib helper
// (unlike Rooted<V>.lift_while on trees), so every jump table repeats LOG + loop.
const LOG = 18
var lift: [0..=LOG, I]?I = none
for i in I: lift[0, i] = next_from(segs[i].r, I.first, I.last)
for k in 1..=LOG:
    for i in I:
        if let mid = lift[k - 1, i]: lift[k, i] = lift[k - 1, mid]

fn count_in(lo: ?I, hi: ?I, ql: Int, qr: Int) -> ?Int:
    if ql > qr: return none
    if lo == none or hi == none: return 0
    let (l, r) = (lo!, hi!)
    if l > r: return 0
    let first = next_from(ql, l, r)
    if first == none: return 0
    var u = first!
    if segs[u].r > qr: return 0
    var cnt = 1
    for k in (0..=LOG).rev:
        if let nu = lift[k, u] and nu <= r and segs[nu].r <= qr:
            u = nu
            cnt += 1 << k
    if let nxt = next_from(segs[u].r, l, r) and segs[nxt].r <= qr:
        cnt += 1
    cnt

type Block = (cr: I, ql: Int, qr: Int, val: Int)
var blocks = Map<I, Block>()
let best = count_in(I.first, I.last, -1, 1e9)!
blocks[I.first] = (I.last, -1, 1e9, best)

var ans: List<Orig> = []
for id in Orig:
    let a = pos[id]
    // FRICTION 38.7: cheat sheet says Map.last_le(k) -> ?I without stating
    // whether Map returns ?K or ?(K, V).
    let cl = blocks.last_le(a)!
    let (cr, ql, qr, val) = blocks[cl]
    let (ml, mr) = (segs[a].l, segs[a].r)
    // FRICTION 38.4: chained `if let ... and let ... and cond` is in decisions.md
    // but missing from docs/one-page-cheat-sheet.md (which only shows single `while let ... and`).
    if let vl = count_in(cl, a - 1, ql, ml) and let vr = count_in(a + 1, cr, mr, qr) and vl + 1 + vr == val:
        ans.push(id)
        blocks.remove(cl)
        if let left_hi = a - 1 and cl <= left_hi:
            blocks[cl] = (left_hi, ql, ml, vl)
        if let right_lo = a + 1 and right_lo <= cr:
            blocks[right_lo] = (cr, mr, qr, vr)

print(ans.len)
print(" ".join("{x}" for x in ans))
