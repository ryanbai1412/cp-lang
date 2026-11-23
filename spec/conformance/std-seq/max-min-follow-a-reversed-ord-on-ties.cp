// expect: ok, output "0 1\n3 2"
// rule: 07.9.3
// `max` and `min` order by the element's own `lt`, here reversed, and keep the first of equal elements; `.rev` keeps the first of the reversed order.
struct P(v: Int, id: Int)
impl Ord for P: fn lt(a, b) = b.v < a.v
let xs = [P(1, 0), P(3, 1), P(3, 2), P(1, 3)]
print(xs.max()!.id, xs.min()!.id)
print(xs.rev.max()!.id, xs.rev.min()!.id)
