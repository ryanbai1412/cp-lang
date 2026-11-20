//! For every position, the fold of all the other positions: prefix folds times suffix folds,
//! for operations with no inverse.

/// `out[i]` is the fold of every element of `xs` except `xs[i]`, in order. O(n).
/// With `+` this is `total - xs[i]`; the point is `gcd`, `min`, `*` modulo a composite,
/// or a tree DP that needs "all children but this one" (rerooting).
use std.algebra.{Monoid}
use std.math.{gcd}
pub fn fold_others<T, M>(xs: List<T>, op: M) -> List<T>:
    if xs.is_empty: return []
    let m = Monoid<T>.of(op)
    let n = xs.len
    var out = List.filled(n, m.id)
    var acc = m.id
    for i in 0..n:
        out[i] = acc
        acc = m.op(acc, xs[i])
    acc = m.id
    for i in (0..n).rev:
        out[i] = m.op(out[i], acc)
        acc = m.op(xs[i], acc)
    out

/// The fold of `xs` as a balanced tree: `op` is applied to neighbours of similar size.
/// For an operation whose cost grows with its operands (polynomial products, merging
/// sorted lists) this is O(log n) rounds instead of a left-leaning chain.
pub fn fold_balanced<T>(xs: List<T>, id: T, op: (T, T) -> T) -> T:
    if xs.is_empty: return id
    var level = xs
    while level.len > 1:
        var next: List<T> = []
        var i = 0
        while i < level.len:
            next.push(if i + 1 < level.len: op(level[i], level[i + 1]) else: level[i])
            i += 2
        level = next
    level[0]

@test
fn every_position_sees_the_rest():
    for _ in 0..40:
        let xs = [rand(1, 40) for _ in 0..rand(0, 7)]
        let g = fold_others(xs, gcd)
        let lo = fold_others(xs, min)
        let cat = fold_others([Str(x) for x in xs], Monoid<Str>("", (a, b) => a + b))
        for i in 0..xs.len:
            var (want, text) = (0, "")
            for j in 0..xs.len:
                if j != i:
                    want = gcd(want, xs[j])
                    text += Str(xs[j])
            assert g[i] == want and cat[i] == text
            // with nothing else to fold, the answer is the identity: `Int.MAX` for `min`
            assert lo[i] == (min(xs[j] for j in 0..xs.len if j != i) ?? Int.MAX)

@test
fn balanced_fold_keeps_the_order():
    for n in 0..12:
        let xs = [Str(i) for i in 0..n]
        var want = ""
        for x in xs: want = want + x
        assert fold_balanced(xs, "", (a, b) => a + b) == want
