// expect: E0606
// rule: 08.2.1
trait Tag:
    fn tag(self) -> Int
struct NoTags
impl Tag for Int:
    fn tag(self) -> Int = self
struct S<F = NoTags>(n: Int)
impl<F: Tag> S<F>:
    fn apply(self, f: F) -> Int = self.n
let t = S(2)
print(t.apply(NoTags())) // <- here
