// expect: ok, output "12 3"
// rule: 08.2.4
// A type parameter that ends a field's range takes one value, and the body reads it as that `Int`.
struct Wheel<const P: Int>(pub v: 0..P):
    fn new(x: Int) -> Wheel<P> = Wheel(x % P)
    fn get(self) -> Int = self.v + P
const K = 3
print(Wheel<7>.new(12).get(), Wheel<K>.new(-3).v + K)
