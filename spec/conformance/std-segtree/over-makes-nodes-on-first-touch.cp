// expect: ok, output "4 0 4\n7 13 13\n1000000520 999999900"
// rule: std.segtree.SegTree.new
// `over: lo..hi` builds a tree over a huge `Int` range: untouched positions hold the identity or `fill`, and nodes are made on first touch.
var s = SegTree<Int, Int>(+, over: 0..1_000_000_000)
s.set(999_999_999, 4)
print(s.all, s.fold(0..999_999_999), s.get(999_999_999))
var lazy = SegTree<Int, Int>(+, act: add, over: -500..1_000_000_000)
lazy.apply(-500..-499, 7)
lazy.set(42, 3)
lazy.at(42) += 10
print(lazy.get(-500), lazy.get(42), lazy.fold(0..100))
lazy.apply_all(1)
print(lazy.all, lazy.fold(100..1_000_000_000))
