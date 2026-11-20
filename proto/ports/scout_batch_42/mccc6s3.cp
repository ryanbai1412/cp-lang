// Mock CCC '24 Contest 1 S3 — Partition Balancing with One Move (CP Lang v2 spec)
// original: corpus/submissions/6103125.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Remove one pivot index i1, splitting into left/right sums, and optionally move
// one element i2 != i1 from the heavier side to the lighter side to minimize
// (imbalance, i1, i2) lexicographically (with no-move i2 = none sorting first).
//
// Note on corpus original: 6103125.cpp declares `next_ind[m]` instead of
// `next_ind[n]` (stack buffer overflow if max(a) + 1 < n) — a bug typed domains
// (`Pos` vs `Val`) reject at compile time — and skips checking `x = (diff + 1) / 2`
// when `diff` is odd and `diff / 2` is also active.

let n: 1..=2e5 = read()
domain Pos = 1..=n
let a: [Pos]Int = read()

// FRICTION MC1: `?Pos` intentionally lacks `Ord`, so minimizing `(diff, i1, i2)`
// where `none` must sort BEFORE every valid `Pos` cannot use tuple `min=`.
type Ans = (diff: Int, i1: Pos, i2: ?Pos)
var best: ?Ans = none

fn consider(diff: Int, i1: Pos, i2: ?Pos):
    let key = (diff, i1, i2 != none, i2 ?? Pos.first)
    if let cur = best:
        if key >= (cur.diff, cur.i1, cur.i2 != none, cur.i2 ?? Pos.first): return
    best = (diff: diff, i1: i1, i2: i2)

fn sweep(arr: [Pos]Int, prefer_first: Bool, emit: (Int, Pos, ?Pos) -> ()):
    var active = Set<Int>()
    var occ = HashMap<Int, Deque<Pos>>()
    for (i, x) in arr.items:
        (occ[x] ??= []).push(i)
        active.add(x)

    var sum_l = 0
    var sum_r = sum(arr)
    for (i, x) in arr.items:
        sum_r -= x
        occ[x].pop_front()
        if occ[x].is_empty: active.remove(x)

        emit(abs(sum_l - sum_r), i, none)
        let diff = sum_r - sum_l
        if diff >= 0:
            for cand in [active.first_ge((diff + 1) / 2), active.last_le(diff / 2)]:
                if let v = cand:
                    let rep = if prefer_first: occ[v].first! else: occ[v].last!
                    emit(abs(diff - 2 * v), i, rep)
        sum_l += x

// FRICTION MC2: Reversing an array and mapping indices back to `Pos` has no
// `i.mirror` / `Pos.rev_at(i)` helper, forcing `.to_idx` + `Pos.from_idx` math.
fn mirror(i: Pos) -> Pos = Pos.from_idx(n - 1 - i.to_idx)!

sweep(a, true, consider)
var rev_a = a.copy()
reverse(mut rev_a)
// FRICTION MC11: Single-expression lambdas (`=> expr`) collide with void-returning
// callbacks that guard an `if let` without `else`, requiring a named nested `fn`.
fn emit_rev(d: Int, i1: Pos, i2: ?Pos):
    if let j = i2: consider(d, mirror(i1), mirror(j))
sweep(rev_a, false, emit_rev)

// FRICTION MC3: `print(opt, none: "-1")` only covers a standalone optional;
// formatting an optional domain value inside an interpolated line needs a branch.
let (i1, i2) = (best!.i1, best!.i2)
if let j = i2: print("{i1} {j}")
else: print("{i1} -1")
