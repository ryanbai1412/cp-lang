// expect: E0602
// A `var` given an unwrapped `lends ?T` keeps its loan until its last use (R05).
struct Box<K>:
    var xs: List<K>
    fn cur(self, i: Int) -> lends ?Cur<K> = if i < xs.len: Some(Cur(self, i)) else: None
    fn key_of(self, c: Cur<K>) -> K = xs[c.id]
    fn add(mut self, x: K): xs.push(x)
pub struct Cur<K>:
    b: Box<K>
    id: Int
var b = Box([3, 4])
var d = b.cur(0)!
b.add(5) // <- here
print(b.key_of(d))
