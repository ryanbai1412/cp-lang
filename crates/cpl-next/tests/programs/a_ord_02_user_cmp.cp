// expect: ok, output "1 1 1 0"
// rule: R08.1.5
struct Frac: p: Int, q: Int
impl Eq for Frac:
    fn eq(self, o: Frac) -> Bool = self.p * o.q == o.p * self.q
impl Ord for Frac:
    fn cmp(self, o: Frac) -> Ordering = (self.p * o.q).cmp(o.p * self.q)
fn b(c: Bool) -> Int = if c: 1 else: 0
print(b(Frac(1, 2) < Frac(2, 3)), b(Frac(2, 4) == Frac(1, 2)), b(Frac(3, 4) > Frac(2, 3)), b(Frac(1, 3) >= Frac(1, 2)))
