// A range query written backwards stops the program: status 3, not an empty answer.
// `l..l` is in order and empty: it is the identity, or nothing to do.
let (n, a, b): (Int, Int, Int) = read()
domain P = 0..n
var st = SegTree<P, Int>(Monoid(0, (x, y) => x + y), [P] i => i.to_idx + 1)
var lz = SegTree<P, Int, Int>(max, [P] i => i.to_idx + 1, act: Action<Int, Int>(0, +, (f, x, _len) => x + f, over: max))
var fw = Fenwick<P, Int>([P] i => i.to_idx + 1)
let (l, r) = (P(a), P(b))
print(st.fold(P(1)..=P(3)), lz.fold(P(1)..=P(3)), fw.fold(P(1)..=P(3)))
print(st.fold(l..l), lz.fold(l..l), fw.fold(l..l))
lz.apply(l..l, 5)
print(lz.fold(P(0)..))
print(st.fold(l..=r))
