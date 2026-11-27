// expect: ok, output "8 5\n1 3 5\n2"
// rule: R08.1.3 R08.2.1
// An inherent impl's fns are methods of its type, beside those in the struct's body; its
// parameters stand for the struct's and its bounds hold of each fn, met where it is called.
trait Tag:
    fn tag(self) -> Int
impl Tag for Int:
    fn tag(self) -> Int = self
struct P:
    x: Int
    fn dbl(self) -> Int = self.x * 2
impl P:
    fn inc(self) -> Int = self.x + 1
struct S<F>(n: Int)
impl<F: Tag> S<F>:
    fn apply(self, f: F) -> Int = self.n
    fn plus<G: Tag>(self, g: G) -> Int = self.n + g.tag()
impl<F> S<F>:
    fn twice(self, f: F) -> Int where F: Tag = f.tag() * 2
let t = S<Int>(1)
print(P(4).dbl(), P(4).inc())
print(t.apply(3), S<Int>(2).plus(1), t.plus(4))
print(t.twice(1))
