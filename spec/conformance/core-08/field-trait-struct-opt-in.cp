// expect: E0606
// rule: 08.1.7
trait HasW: w: Int
fn weight<T: HasW>(x: T) -> Int = x.w
struct Road: w: Int
print(weight(Road(4))) // <- here
