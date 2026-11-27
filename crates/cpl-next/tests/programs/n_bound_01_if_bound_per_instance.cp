// expect: ok, output "fancy plain\n1 0\n5 0\n1 2 0"
// `if T: Tr<A>:` is decided per instance at compile time, its trait args included; the branch
// it guards sees the bound, and is never built for an instance that fails it (R08.2.9).
trait Fancy
struct A: x: Int
struct B: x: Int
impl Fancy for A
fn kind<T>(x: T) -> Str:
    if T: Fancy:
        return "fancy"
    return "plain"
trait Named:
    @get fn name(self) -> Int
struct S: n: Int
impl Named for S:
    @get fn name(self) -> Int = self.n
fn name_or_zero<X>(x: X) -> Int:
    if X: Named:
        return x.name
    0
trait Pick<T>:
    fn pick(self, a: T, b: T) -> Bool
struct Hi
impl Pick<Int> for Hi:
    fn pick(self, a: Int, b: Int) -> Bool = a > b
struct Lo
impl Pick<F64> for Lo:
    fn pick(self, a: F64, b: F64) -> Bool = a < b
fn first<T, M>(m: M, a: T, b: T) -> Int:
    if M: Pick<T>:
        return if m.pick(a, b): 1 else: 2
    0
fn picks<T, M>(m: M, a: T) -> Int = if M: Pick<T>: 1 else: 0
print(kind(A(1)), kind(B(1)))
print(picks(Hi(), 1), picks(Lo(), 1))
print(name_or_zero(S(5)), name_or_zero(3))
print(first(Hi(), 3, 2), first(Hi(), 1, 2), first(Lo(), 1, 2))
