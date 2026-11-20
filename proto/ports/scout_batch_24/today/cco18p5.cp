// Runnable port (proto/cp.py lib-run) of statement-first cco18p5.cp
// original: corpus/submissions/4801867.cpp

let (n, k, q): (Int, Int, Int) = read()
domain I = 1..=n
let init_a: [I]Int = read()
domain Q = 0..q
let upds: [Q](pos: I, x: Int) = read()

struct Peak:
    has: Bool
    val: Int
    idx: Int

let empty_peak = Peak(false, 0, -1)

let arg_max = Monoid<Peak>(
    id: empty_peak,
    op: (a, b) =>
        if not a.has: b
        else: if not b.has: a
        else: if a.val > b.val or (a.val == b.val and a.idx > b.idx): a else: b
)

let init_peaks = [I] i => Peak(true, init_a[i], i.to_idx)
var val_st = SegTree<I, Peak>(arg_max, init_peaks)
var pair_st = SegTree<I, Int>(max)

fn left_best(p: I) -> Peak:
    let pi = p.to_idx
    if pi == 0: return empty_peak
    let lo = max(0, pi - k + 1)
    val_st.fold(I.nth(lo)..p)

fn right_best(p: I) -> Peak:
    let pi = p.to_idx
    if pi + 1 >= n: return empty_peak
    let hi = min(n - 1, pi + k - 1)
    val_st.fold(I.nth(pi + 1)..=I.nth(hi))

fn refresh(p: I):
    let me = val_st.get(p)
    let nb = arg_max.op(left_best(p), right_best(p))
    if nb.has and (me.val > nb.val or (me.val == nb.val and me.idx > nb.idx)):
        pair_st.set(p, me.val + nb.val)
    else:
        pair_st.set(p, 0)

for p in I: refresh(p)
print(pair_st.fold(I.first..=I.last))

for qi in Q:
    let pos = upds[qi].pos
    let x = upds[qi].x
    let old_l = left_best(pos)
    let old_r = right_best(pos)
    val_st.set(pos, Peak(true, x, pos.to_idx))
    refresh(pos)
    if old_l.has: refresh(I.nth(old_l.idx))
    if old_r.has: refresh(I.nth(old_r.idx))
    print(pair_st.fold(I.first..=I.last))
