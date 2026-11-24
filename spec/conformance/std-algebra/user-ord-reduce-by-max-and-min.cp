// expect: ok, output "2 3"
// rule: 07.9.2
// `reduce(max)` folds a user `Ord` type, which has no ends to be `fold(max)`'s identity.
struct Frac: p: Int, q: Int
impl Ord for Frac:
    fn cmp(self, o: Frac) -> Ordering = (self.p * o.q).cmp(o.p * self.q)
let xs = [Frac(1, 2), Frac(2, 3), Frac(1, 3)]
print(xs.reduce(max)!.p, xs.reduce(min)!.q)
