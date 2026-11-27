// expect: ok, output "3 5 4 2\n8 3 4 4\n3 6"
// rule: R02.9.3
// A result's `_` in a struct's domain parameter is a domain of its own, made by the body; each
// value a caller binds owns one, so two values' positions never mix. A `_` in any other parameter
// is what the body's value has there.
struct Tagged<V, P>:
    at: [P]V
    fn size(self) -> Int = P.count
    fn last(self) -> V = at[P.nth(P.count - 1)]
fn make<V: Domain>(from: V, k: Int) -> Tagged<V, _>:
    domain O = 0..k
    Tagged<V, O>(at: [O] o => V.nth(from.ord + o.ord))
fn again<V: Domain>(from: V, k: Int) -> Tagged<V, _> = make(from, k)
fn pair<V: Domain>(from: V, k: Int) -> (t: Tagged<V, _>, n: Int) = (t: make(from, k), n: k)
struct Box3<V, P, A>:
    at: [P]V
    tag: A
fn boxed<V: Domain>(from: V, k: Int) -> Box3<V, _, _>:
    domain O = 0..k
    Box3<V, O, Int>(at: [O] o => V.nth(from.ord + o.ord), tag: 7)
fn maybe<V: Domain>(from: V, k: Int) -> ?Box3<V, _, Int>:
    if k == 0: return None
    domain O = 0..k
    Some(Box3<V, O, Int>(at: [O] _ => from, tag: k))
domain W = 0..10
let a = make(W(1), 3)
let b = again(W(0), 5)
print(a.size(), b.size(), Int(b.last()), Int(a.at[0]) + 1)
let c = boxed(W(1), 3)
let m = maybe(W(2), 4)!
print(c.tag + 1, c.at.len, m.tag, m.at.len)
let (t, n) = pair(W(1), 3)
var s = 0
for p in t.P: s += Int(t.at[p])
print(n, s)
