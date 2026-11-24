// expect: ok, output "1097 1098 5 1 2 3"
// rule: 08.1.9
// A trait's `@get` that an impl writes for a built-in type, or a blanket `impl<D: Domain> .. for D`
// writes for every domain, is read as a member, in a generic body through its bound and on a value
// of that type, the same as a struct's.
pub trait Tok:
    @get fn tk(self) -> Int = 0
impl Tok for Char:
    @get fn tk(self) -> Int = Int(self) + 1000
impl Tok for Int:
    @get fn tk(self) -> Int = self
impl Tok for Bool:
    @get fn tk(self) -> Int = if self: 1 else: 0
impl<D: Domain> Tok for D:
    @get fn tk(self) -> Int = self.ord
domain V = 0..4
fn tok<T: Tok>(x: T) -> Int = x.tk
print(tok('a'), 'b'.tk, tok(5), true.tk, tok(V(2)), V(3).tk)
