// expect: ok, output "0"
domain V = 0..5
let u: SegTree<V, Int, Int> = SegTree(max, [V] v => 0)
print(u.all)
