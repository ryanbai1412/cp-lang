// expect: ok, output "2 1"
// rule: 08.1.2
// An impl for every domain that writes no body gets the trait's default getter, bound by the impl's trait arguments, directly and through a bound.
pub trait Hop<V>:
    @get fn dest(self) -> V = self
impl<D: Domain> Hop<D> for D
domain V = 0..3
fn d<X: Hop<V>>(x: X) -> V = x.dest
print(V(2).dest, d(V(1)))
