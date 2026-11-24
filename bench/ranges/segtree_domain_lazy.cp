// Emitted size: a lazy SegTree over a domain writes only the dense tree (atcoder_dp_w).
let n: 1..=2e5 = read()
domain P = 0..=n
var t = SegTree<P, Int, Int>(max, fill: 0, act: add)
t.apply(.., 1)
print(t.fold(..))
