// expect: E0606
// rule: 07.9.2
// A user `Ord` type has no `MIN` or `MAX`, so `max` has no identity on it and `fold(max)` is an
// error; `reduce(max)` is the fold without one.
struct Frac: p: Int, q: Int
impl Ord for Frac:
    fn cmp(self, o: Frac) -> Ordering = (self.p * o.q).cmp(o.p * self.q)
let xs = [Frac(1, 2), Frac(2, 3), Frac(1, 3)]
print(xs.reduce(max)!.p, xs.reduce(min)!.q)
print(xs.fold(max).p)  // <- here
