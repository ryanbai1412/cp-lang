// expect: E0606
// rule: 08.2.1
trait Tag:
    fn tag(self) -> Int
struct NoTags
impl Tag for Int:
    fn tag(self) -> Int = self
struct S<F = NoTags>(n: Int)
impl<F> S<F>:
    fn apply(self, f: F) -> Int where F: Tag = self.n
print(S(2).apply(NoTags())) // <- here
