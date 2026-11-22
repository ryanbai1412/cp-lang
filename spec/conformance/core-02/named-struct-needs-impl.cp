// expect: E0606 at 5
// rule: 02.6.5
domain V = 0..3
fn dest<A: Target<V>>(a: A) -> V = a.to
struct Arc: to: V, w: Int
print(Int(dest((to: V(1), w: 7))))
print(Int(dest(Arc(V(2), 5))))
