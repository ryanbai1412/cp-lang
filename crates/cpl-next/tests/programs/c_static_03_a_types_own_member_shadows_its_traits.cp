// expect: ok, output "7\n7\n0\n0"
// rule: R08.1.4
// A type's own member of a name shadows the same-named one of a trait it implements, called or read: `N.zero` is N's, `Zero.zero` at N is the impl's.
struct N(pub v: Int):
    @get fn zero() -> N = N(7)
    fn one() -> N = N(7)
impl Zero for N:
    @get fn zero() -> N = N(0)
impl One for N:
    @get fn one() -> N = N(0)
fn z<T: Zero>() -> T = T.zero
fn o<T: One>() -> T = T.one
print(N.zero.v)
print(N.one().v)
print(z<N>().v)
print(o<N>().v)
