// expect: ok, output "8 30"
// rule: 08.1.4
// An impl for a table type or a list type is filed by the type's head, so a bound on the
// trait takes a value of it and a method call on one reaches the impl.
trait At<T, U>:
    fn at_key(self, x: T) -> U
impl At<Int, Int> for List<Int>:
    fn at_key(self, x: Int) -> Int = self[x]
impl<V> At<V, Int> for [V]Int:
    fn at_key(self, x: V) -> Int = self[x] * 10
fn read_at<B: At<T, U>, T, U>(b: B, x: T) -> U = b.at_key(x)
enum E:
    A
    B
let t: [E]Int = [1, 3]
print(read_at([7, 8], 1), read_at(t, E.B))
