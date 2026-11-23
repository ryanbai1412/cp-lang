// A trait bound on the parameter a generic call is passed to binds that call's generics before its
// lambdas are probed: `St: Feed<T>` with `T` known makes `acc`'s `x` a `(w: Int)`.
trait Feed<T>:
    fn feed(mut self, x: T)
struct Acc<S, T>(pub state: S, add: (mut S, T) -> ())
impl Feed<T> for Acc<S, T>:
    fn feed(mut self, x: T):
        add(mut state, x)
fn acc<S, T>(state: S, add: (mut S, T) -> ()) -> Acc<S, T> = Acc(state, add)
fn run<T, St: Feed<T>>(xs: List<T>, st: St) -> St:
    var s = st
    for x in xs: s.feed(x)
    s
print(run([(w: 1), (w: 2), (w: 4)], acc(0, (mut s, x) => s += x.w)).state)
print(run(["ab", "c"], acc(10, (mut s, x) => s -= x.len)).state)
