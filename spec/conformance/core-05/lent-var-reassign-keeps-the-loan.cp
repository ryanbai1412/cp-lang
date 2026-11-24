// expect: E0602
// A `var` holding a lent value keeps the loan of each value assigned to it until its last use,
// as the `let` form does: `c = b.next(c)` re-takes `b`, so changing `b` before `c` is printed is
// rejected (R05).
struct Box:
    var xs: List<Int>
    fn cur(self, i: Int) -> lends Cur = Cur(self, i)
    fn next(self, c: Cur) -> lends Cur = Cur(self, c.id + 1)
    fn key(self, c: Cur) -> Int = xs[c.id]
    fn add(mut self, x: Int): xs.push(x)
pub struct Cur:
    b: Box
    id: Int
var b = Box([1, 2, 3])
var c = b.cur(0)
c = b.next(c)
b.add(4) // <- here
print(c.id)
