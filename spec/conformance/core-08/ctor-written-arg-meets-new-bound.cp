// expect: E0606
// rule: 08.2.5
// A written type argument fixes `F` but not past `new`'s bound: `S<Int, NoTags>(1, act: Add())` needs `Add: Acts<NoTags>`, and `Add`'s only impl is `Acts<Int>`.
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
