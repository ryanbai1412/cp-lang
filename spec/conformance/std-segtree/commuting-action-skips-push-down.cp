// expect: ok, output "300 300"
// rule: std.segtree SegTree.apply with a commuting Action (`commutes: true`, or stock `add`) skips push-down and keeps every answer
domain I = 0..13
// min with its count, under range add: the shape of a "how many positions are uncovered" tree
let act = Action<(Int, Int), Int>(
    id: 0,
    compose: (f, g) => f + g,
    apply: (f, x, _) => (x.0 + f, x.1),
    over: Monoid(id: (1_000_000, 0), op: (a, b) => if a.0 < b.0: a elif b.0 < a.0: b else: (a.0, a.1 + b.1)),
    commutes: true,
)
var a = [I] _ => 0
var st = SegTree<I, (Int, Int), Int>(act.over, [I] _ => (0, 1), act:)
var sums = SegTree(+, [I] _ => 0, act: add)
var seed = 7
fn next(mut seed: Int, n: Int) -> Int:
    seed = (seed * 1_103_515_245 + 12_345) % 2_147_483_648
    seed % n
var (good, good_sums) = (0, 0)
for _ in 0..300:
    let l = I.nth(next(mut seed, 13))
    let r = I.nth(next(mut seed, 13 - l.ord) + l.ord)
    let d = next(mut seed, 7) - 3
    for i in l..=r: a[i] += d
    st.apply(l..=r, d)
    sums.apply(l..=r, d)
    let ql = I.nth(next(mut seed, 13))
    let qr = I.nth(next(mut seed, 13 - ql.ord) + ql.ord)
    let lo = min(a[i] for i in ql..=qr)!
    if st.fold(ql..=qr) == (lo, count(a[i] == lo for i in ql..=qr)): good += 1
    if sums.fold(ql..=qr) == sum(a[i] for i in ql..=qr): good_sums += 1
print(good, good_sums)
