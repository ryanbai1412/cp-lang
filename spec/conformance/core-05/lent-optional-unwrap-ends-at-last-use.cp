// expect: ok, output "3\n4\n4"
// A lend taken by `!` or `if let` from a `lends ?T` ends at its last use, so `b` may change after (R05).
struct Box<K>:
    var xs: List<K>
    fn cur(self, i: Int) -> lends ?Cur<K> = if i < xs.len: Some(Cur(self, i)) else: None
    fn key_of(self, c: Cur<K>) -> K = xs[c.id]
    fn add(mut self, x: K): xs.push(x)
pub struct Cur<K>:
    b: Box<K>
    id: Int
var b = Box([3, 4])
let d = b.cur(0)!
print(b.key_of(d))
b.add(5)
if let e = b.cur(1):
    print(b.key_of(e))
b.add(6)
print(b.xs.len)
