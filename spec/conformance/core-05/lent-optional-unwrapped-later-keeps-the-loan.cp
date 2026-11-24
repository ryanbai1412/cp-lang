// expect: E0602
// Unwrapping a `let` holding a `lends ?T` keeps the loan the optional took (R05).
struct Box<K>:
    var xs: List<K>
    fn cur(self, i: Int) -> lends ?Cur<K> = if i < xs.len: Some(Cur(self, i)) else: None
    fn key_of(self, c: Cur<K>) -> K = xs[c.id]
    fn add(mut self, x: K): xs.push(x)
pub struct Cur<K>:
    b: Box<K>
    id: Int
var b = Box([3, 4])
let o = b.cur(0)
let d = o!
b.add(5) // <- here
print(b.key_of(d))
