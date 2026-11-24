// expect: ok, output "-10\n-10 3\n-5"
// `SegTree<D, T>(max)` over a range alias starts each empty fold at `T`'s low end (-10 here), not
// `Int`'s, so arithmetic on an empty fold stays in range (cdde42a75).
type M = -10..=10
var s = SegTree<0..4, M>(max)
print(s.fold(0..2))
s[1] = 3
print(s.fold(2..4), s.fold(0..4))
print(s.fold(0..0) + 5)
