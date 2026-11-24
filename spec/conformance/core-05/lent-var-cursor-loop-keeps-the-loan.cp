// expect: E0602
// A `var` walked with `while let` holds the loan of each cursor stored in it for the whole loop,
// so changing the source between two steps is rejected (R05).
struct Box:
    var xs: List<Int>
    fn cur(self, i: Int) -> lends ?Cur = if i < xs.len: Some(Cur(self, i)) else: None
    fn next_cursor(self, c: Cur) -> lends ?Cur = if c.id + 1 < xs.len: Some(Cur(self, c.id + 1)) else: None
    fn add(mut self, x: Int): xs.push(x)
pub struct Cur:
    b: Box
    id: Int
fn f():
    var b = Box([1, 2, 3])
    var c = b.cur(0)
    while let x = c:
        print(x.id)
        b.add(9) // <- here
        c = b.next_cursor(x)
f()
