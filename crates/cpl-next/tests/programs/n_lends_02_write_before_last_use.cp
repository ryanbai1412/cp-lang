// expect: E0602
// rule: R05.6.1, R05.7.6
// Writing the source of a `lends` result before the result's last use is rejected.
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
b.add(9) // <- here
print(key(c))
