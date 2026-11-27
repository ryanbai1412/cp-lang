// expect: ok, output "11 12"
// rule: R08.1.2
// A struct that holds no `S` is one type whether its `S` was bound from a lent value or not.
struct Tag<S>(n: Int)
struct Within<S>(store: S, t: Tag<S>)
struct Bag(xs: List<Int>)
impl<S> Tag<S>:
    fn within(self, store: S) -> Within<S> = Within(store, self)
let b = Bag([1, 2])
let t = Tag<Bag>(10)
let w = t.within(b)
print(w.t.n + w.store.xs[0], w.t.n + w.store.xs[1])
