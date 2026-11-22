// expect: E0606 at 4
// rule: 08.1.7
trait HasW: w: Int
fn weight<T: HasW>(x: T) -> Int = x.w
struct Road: w: Int
print(weight(Road(4)))
