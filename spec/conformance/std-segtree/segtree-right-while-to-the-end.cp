// expect: ok, output "YES"
// rule: std.segtree.SegTree.right_while
domain D = 0..4
let s = SegTree<D, Int>(+, fill: 1)
let first = D.cuts.first!
let last = D.cuts.last!
print(yes(s.right_while(first, x => x <= 10) == last and s.right_while(last, x => x <= 0) == last))
