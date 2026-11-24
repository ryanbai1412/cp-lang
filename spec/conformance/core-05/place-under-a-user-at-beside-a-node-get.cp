// expect: ok, output "15 5 2 3 3 9"
// rule: 05.6.3
// Beside a `get -> Node`, each place under `p[k]` (a `mut self` receiver, a `mut` argument, an assignment, a `push` on a field) changes the slot the `lends mut` `at` lends, not a copy `get` returns.
struct C:
    var n: Int
    fn bump(mut self, by: Int):
        n += by
fn inc(mut c: C):
    c.n += 10
struct Node:
    var c: C
    var kids: List<Int>
struct Arena:
    _xs: List<Node>
    pub fn at(mut self, i: Int) -> lends mut Node:
        lend mut _xs[i]
    pub fn get(self, i: Int) -> Node = _xs[i]
var p = Arena([Node(C(1), [2]), Node(C(7), [])])
p[0].c.bump(4)
p[1].c.n = 2
p[1].c.n += 3
inc(mut p[0].c)
p[0].kids.push(3)
p[0].kids[0] += 1
p[1].kids.push(9)
print(p[0].c.n, p[1].c.n, p[0].kids.len, p[0].kids[0], p[0].kids[1], p[1].kids[0])
