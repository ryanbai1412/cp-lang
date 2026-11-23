// `x.cmp(y)` is the Ord method on every Ord type (spec/core/08-traits-generics.md).
struct Frac: p: Int, q: Int
impl Ord for Frac:
    fn cmp(self, o: Frac) -> Ordering = (self.p * o.q).cmp(o.p * self.q)
let xs = [Frac(1, 2), Frac(1, 3)].sorted()
print(xs[0].p, xs[0].q, xs[1].p, xs[1].q, yes(Frac(2, 3) > Frac(1, 2)))
print(yes(cmp(1, 2) == Ordering.Less), yes(3.cmp(3) == Ordering.Equal), yes("b".cmp("a") == Ordering.Greater))
