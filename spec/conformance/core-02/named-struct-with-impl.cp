// expect: ok, output "2"
// rule: 02.6.5
domain V = 0..3
fn dest<A: Target<V>>(a: A) -> V = a.to
struct Arc: to: V, w: Int
impl Target<V> for Arc: fn to(self) -> V = self.to
print(Int(dest(Arc(V(2), 5))))
