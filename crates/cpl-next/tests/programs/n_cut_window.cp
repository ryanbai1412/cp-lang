// expect: ok, output "2 3 4\n1 3 1\n2 2"
// rule: R04.3.1
// `v.l(k)` is the left cut of the `k` positions ending at `v` and `v.r(k)` the right cut of the
// `k` positions from `v`; with no `k` they are `v.l` and `v.r`. A struct bounding its domain
// parameter `Ordered` reads them, as std.tree's `sub` does with a subtree's size.
struct Win<P: Ordered>:
    at: P
    pub fn span(self, k: Int) -> Range<P> = at.l..at.r(k)
    pub fn back(self, k: Int) -> Range<P> = at.l(k)..at.r
domain V = 1..=6
let w = Win(V(2))
print(w.span(2).len, w.span(3).len, w.back(4).len)
print(Int(V(2).l), Int(V(3).r), Int(V(3).l(2)))
print(List(w.span(2))[1].ord, Int(V(1).r(2)))
