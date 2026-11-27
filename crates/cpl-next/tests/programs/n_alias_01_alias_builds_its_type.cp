// expect: ok, output "5 7\n0\n3\n42"
// Calling a type alias calls the type it names: its `new` when it has one, its fields otherwise,
// and a conversion for a core type.
struct P(x: Int, y: Int)
type Q = P
let q = Q(5, 7)
print(q.x, q.y)
type G<T> = List<T>
print(G<Int>().len)
type I = Int
print(I('c') - 96)
struct Boxed<T>(val: T)
type B<T> = Boxed<T>
print(B(42).val)
