// expect: ok, output "3 4"
// rule: 08.1.7
trait HasW: w: Int
fn weight<T: HasW>(x: T) -> Int = x.w
struct Road: w: Int, len: Int
impl HasW for Road
print(weight((w: 3, z: 1)), weight(Road(4, 9)))
