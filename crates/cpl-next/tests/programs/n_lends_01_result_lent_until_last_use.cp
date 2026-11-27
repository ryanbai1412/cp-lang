// expect: ok, output "3\n9"
// rule: R05.6.1, R05.7.6
// A `lends` result is an ordinary value: passed to a fn, it lends its source until that last use,
// so writing the source after it is fine.
struct Box:
    var xs: List<Int>
    fn view(self, i: Int) -> lends Cur = Cur(self, i)
    fn add(mut self, x: Int): xs.push(x)
pub struct Cur:
    b: Box
    id: Int
fn key(c: Cur) -> Int = c.b.xs[c.id]
var b = Box([1, 2, 3])
let c = b.view(2)
print(key(c))
b.add(9)
print(key(b.view(3)))
