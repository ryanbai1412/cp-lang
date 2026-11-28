// expect: E0606
// rule: R08.2.2
// `T.from_int` on a `T` whose bounds give no `from_int` names the missing bound, not a domain.
struct P<T: Eq>(v: T):
    fn of(x: Int) -> P<T> = P(T.from_int(x))
print(1)
