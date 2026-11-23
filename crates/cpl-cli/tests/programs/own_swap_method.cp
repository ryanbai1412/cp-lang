// A struct's own `swap(self)` is not the list mutator of that name, so a
// `let` receiver may call it, and `Pair(second, first)` in its body builds
// the `Pair<B, A>` it returns (R05.4.2).
domain City = 1..=3
struct Pair<A, B>(first: A, second: B):
    fn swap(self) -> Pair<B, A> = Pair(second, first)
let p = Pair(City.first!, 'x')
let q = p.swap()
print(p.first.label, q.first, q.second.label)
