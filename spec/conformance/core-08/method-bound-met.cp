// expect: ok, output "1 3 5"
// rule: 08.2.1
trait Tag:
    fn tag(self) -> Int
struct NoTags
impl Tag for Int:
    fn tag(self) -> Int = self
struct S<F = NoTags>(n: Int)
impl<F: Tag> S<F>:
    fn apply(self, f: F) -> Int = self.n
    fn plus<G: Tag>(self, g: G) -> Int where F: Tag = self.n + g.tag()
let t = S<Int>(1)
print(t.apply(3), S<Int>(2).plus(1), t.plus(4))
