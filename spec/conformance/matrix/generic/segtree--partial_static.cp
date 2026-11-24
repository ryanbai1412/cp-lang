// expect: ok, output "0"
domain V = 0..5
let t = SegTree<V, Int>.of(max, v => 0)
let u: SegTree<V, Int, Int> = t
print(u.all)
