// expect: ok, output "ok 600"
// A lazy SegTree's reads (`get`, `fold`, `right_while`, `left_while`) compose the pending tags on
// the way down instead of pushing them, so they work on a tree that is not `mut`, even with tags
// that don't commute.
domain D = 0..13

let aff = Action<Int, (Int, Int)>(
    id: (1, 0),
    compose: (f, g) => (f.0 * g.0, f.0 * g.1 + f.1),
    apply: (f, x, n) => f.0 * x + f.1 * n,
)

fn sum_agrees(st: SegTree<D, Int, (Int, Int)>, w: [D]Int) -> Bool:
    for i in D:
        if st.get(i) != w[i]: return false
    for l in 0..=13:
        for h in l..=13:
            let want = sum(w[D(i)] for i in l..h)
            if h > l and st.fold(D(l)..=D(h - 1)) != want: return false
    true

fn max_agrees(st: SegTree<D, Int, ?Int>, w: [D]Int, x: Int) -> Bool:
    for l in 0..=13:
        var r = l
        while r < 13 and w[D(r)] < x: r += 1
        if st.right_while(D.cuts.nth(l), m => m < x) != D.cuts.nth(r): return false
        var s = l
        while s > 0 and w[D(s - 1)] < x: s -= 1
        if st.left_while(D.cuts.nth(l), m => m < x) != D.cuts.nth(s): return false
    true

var a: [D]Int = [D] _ => rand(-5, 5)
var b: [D]Int = [D] _ => rand(0, 20)
var sa = SegTree<D, Int, (Int, Int)>(+, act: aff, a: a)
var sb = SegTree<D, Int, ?Int>(max, act: assign, a: b)
var ok = 0
for _ in 0..600:
    let l = rand(0, 12)
    let h = rand(l + 1, 13)
    let f = (if rand(0, 1) == 0: 1 else: -1, rand(-3, 3))
    sa.apply(D(l)..=D(h - 1), f)
    for i in l..h: a[D(i)] = f.0 * a[D(i)] + f.1
    let v = rand(0, 20)
    sb.apply(D(l)..=D(h - 1), Some(v))
    for i in l..h: b[D(i)] = v
    if sum_agrees(sa, a) and max_agrees(sb, b, rand(0, 21)): ok += 1
print("ok", ok)
