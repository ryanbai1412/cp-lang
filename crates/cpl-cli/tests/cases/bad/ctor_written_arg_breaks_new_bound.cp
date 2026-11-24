// error: `Add` does not implement `Acts<NoTags>` required by `S<Int, NoTags>`
trait Acts<F>:
    fn tag(self) -> F
struct NoTags
struct NoAct
impl Acts<NoTags> for NoAct:
    fn tag(self) -> NoTags = NoTags()
struct Add
impl Acts<Int> for Add:
    fn tag(self) -> Int = 0
struct S<D, F = NoTags>(n: Int, tag: F):
    pub fn new<A: Acts<F>>(n: Int, act: A = NoAct()) -> S<D, F> = S<D, F>(n, act.tag())
    pub fn apply(self, f: F) -> F = f
let b = S<Int, NoTags>(1, act: Add())
print(b.n)
