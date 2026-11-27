// expect: ok, output "1 2 3"
// rule: 08.1.6
// engines: next
// A derived impl asks of the struct's parameters what its fields' impls ask: `E` sits inside a
// record inside a `List`, so `Walk<V, E>`'s `Eq`, `Ord` and `Clone` each ask `E` for theirs. A
// struct overloaded by arity derives each impl for its own fields (R02.11.4). Interpreter and
// C++ agree.
struct Walk<V, E>:
    start: V
    arcs: List<(to: V, e: E)>
struct Pair<T>:
    v: T
struct Pair<A, B>:
    a: A
    b: B
fn same<V: Eq + Clone, E: Eq + Clone>(w: Walk<V, E>) -> Bool = w == w.clone()
let w = Walk(1, [(to: 2, e: 3)])
assert same(w) and not (w < w)
let p = Pair<Int, Int>(a: 1, b: 2)
let q = Pair<Int>(v: 3)
assert p == p.clone() and not (q < q)
print(w.arcs.len, p.b, q.v)
