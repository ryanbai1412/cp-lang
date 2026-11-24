// expect: E0606
// rule: 08.2.1
trait Tag:
    fn tag(self) -> Int
struct NoTags
impl Tag for Int:
    fn tag(self) -> Int = self
struct S(n: Int)
impl S:
    fn apply<G: Tag>(self, g: G) -> Int = self.n
print(S(2).apply(NoTags())) // <- here
