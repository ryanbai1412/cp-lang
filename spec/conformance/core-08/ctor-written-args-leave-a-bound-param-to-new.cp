// expect: ok, output "5 0"
// rule: 08.2.5
// `S<Int>(1, act: Add())` writes `D` alone: `new`'s bound `A: Acts<F>` binds `F = Int` from `Add`'s impl before `F`'s default `NoTags` fills it, as with no type arguments written.
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
let a = S<Int>(1, act: Add())
let b = S<Int>(1)
print(a.apply(5), a.tag)
let _ = b.apply(NoTags())
