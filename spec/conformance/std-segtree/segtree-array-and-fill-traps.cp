// expect: R0401 (checked)
// rule: std.segtree.SegTree.new
domain D = 0..2
let a: [D]Int = [D] v => Int(v)
let s = SegTree<D, Int>(+, a, fill: 0) // <- here
print(s.all)
