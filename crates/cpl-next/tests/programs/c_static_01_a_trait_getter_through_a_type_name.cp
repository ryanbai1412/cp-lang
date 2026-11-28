// expect: ok, output "3\n0"
// rule: R08.2.8
// A type name reaches a self-less `@get` fn of a trait it implements without `()`, in its own fns too.
struct M(pub v: Int):
    fn both(a: M, b: M) -> M = M(a.v + b.v + M.zero.v)
impl Zero for M:
    @get fn zero() -> M = M(0)
print(M.both(M(1), M(2)).v)
print(M.zero.v)
