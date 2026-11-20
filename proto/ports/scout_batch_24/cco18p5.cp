// CCO '18 P5 - Boring Lectures (DMOJ: cco18p5, 25pt, TL 2.5s, ML 512MB)
// Statement-first CP Lang solution (written before reading corpus C++).
// (I/O verified against corpus/submissions/4801867.cpp: updates are plain (pos, x),
// not XOR-encrypted, so `pos: I` is directly typed.)
//
// Array a[1..=n] (2 <= k <= n <= 1e6) with q <= 5e5 point updates (pos, x): set a[pos] = x.
// Initially and after each update, print max_{1 <= u < v <= n, v - u < k} (a[u] + a[v]).
//
// Local-peak theorem: some optimal pair contains a window-radius-(k-1) local peak `p`
// (tie-broken by index) paired with its best neighbor in [p - k + 1 .. p + k - 1] \ {p}.
// On updating position `i`, the ONLY positions whose local-peak status or best neighbor
// can change are `i`, `L = argmax(a[i - k + 1 .. i - 1])`, and `R = argmax(a[i + 1 .. i + k - 1])`.
// Two `SegTree`s: `val_st` (argmax `(val, idx)`) and `pair_st` (max valid peak sum).
use std.algebra.{Monoid, Max}
use std.segtree.{SegTree}

let (n, k, q): (2..=1e6, 2..=n, 0..=5e5) = read()
domain I = 1..=n
let init_a: [I]Int = read()
domain Q = 0..q
struct Upd(pos: I, x: Int)
let upds: [Q]Upd = read()

// WISH A14: `stdlib/segtree.cp` has `Max<Int>`, which returns only the max value,
// not `(val, pos: I)` (ArgMax). Every argmax segment tree repeats this 6-line
// optional-pair `Monoid` instead of writing `ArgMax<I, Int>`.
struct Peak(val: Int, pos: I)
const ArgMax = Monoid<?Peak>(
    id: none,
    op: (a, b) =>
        if a == none: b
        else: if b == none: a
        else: if a!.val > b!.val or (a!.val == b!.val and a!.pos.to_idx > b!.pos.to_idx): a else: b
)

// WISH A15: Clamping a window `[p - k + 1 .. p - 1]` around a domain index `p: I`
// drops out of domain types into `.to_idx` arithmetic + `I.nth(...)` in four places.
// A domain helper `p.window(rad)` or `I.clamp_range(lo_idx, hi_idx)` would remove all
// manual boundary clamping (`max(0, pi - k + 1)` / `min(n - 1, pi + k - 1)`).
fn solve() -> List<Int>:
    var val_st = SegTree<I, ?Peak>.from(ArgMax, [I] i => Peak(init_a[i], i))
    var pair_st = SegTree<I, Int>.new(Max<Int>)

    fn left_best(p: I) -> ?Peak:
        let pi = p.to_idx
        if pi == 0: return none
        let lo = max(0, pi - k + 1)
        val_st.fold(I.nth(lo)..p)

    fn right_best(p: I) -> ?Peak:
        let pi = p.to_idx
        if pi + 1 >= n: return none
        let hi = min(n - 1, pi + k - 1)
        val_st.fold(I.nth(pi + 1)..=I.nth(hi))

    mut fn refresh(p: I):
        let me = val_st.get(p)!
        let lb = left_best(p)
        let rb = right_best(p)
        let best_nbr = ArgMax.op(lb, rb)
        if let nb = best_nbr:
            let better = me.val > nb.val or (me.val == nb.val and me.pos.to_idx > nb.pos.to_idx)
            pair_st.set(p, if better: me.val + nb.val else: 0)
        else:
            pair_st.set(p, 0)

    for p in I: refresh(p)

    var out: List<Int> = [pair_st.all]

    for qi in Q:
        let pos = upds[qi].pos
        let x = upds[qi].x
        let old_l = left_best(pos)
        let old_r = right_best(pos)
        val_st.set(pos, Peak(x, pos))
        refresh(pos)
        if let lp = old_l: refresh(lp.pos)
        if let rp = old_r: refresh(rp.pos)
        out.push(pair_st.all)
    out

@brute(solve)
fn solve_brute() -> List<Int>
    where n <= 100 and q <= 100:
    var cur = init_a.copy()
    fn calc() -> Int:
        var best = 0
        for u in 0..n:
            for v in (u + 1)..min(n, u + k):
                best max= cur[I.nth(u)] + cur[I.nth(v)]
        best
    var out = [calc()]
    for qi in Q:
        cur[upds[qi].pos] = upds[qi].x
        out.push(calc())
    out

for ans in solve(): print(ans)
