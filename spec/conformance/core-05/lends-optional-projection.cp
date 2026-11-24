// expect: ok, output "0"
// `lends ?T` returns `Some` of a lend of `self` or `None`: the lend sources of the arms join,
// and `None` lends nothing (R05).
struct Box:
    var xs: List<Int>
    fn cur(self, i: Int) -> lends ?Cur = if i < xs.len: Some(Cur(self, i)) else: None
    fn add(mut self, x: Int): xs.push(x)
pub struct Cur:
    b: Box
    id: Int
var b = Box([1, 2, 3])
let c = b.cur(0)
print(c!.id)
b.add(4)
