// expect: ok, output "15 5 20 7 0"
// rule: 05.6.3
// Beside a `get -> Int`, each place under `p[k]` (a `mut self` receiver, a `mut` argument, an assignment, a `push` or `swap` on a field) is the slot the `lends mut` `at` lends, as the checker reads it.
struct C:
    var n: Int
    fn bump(mut self, by: Int):
        n += by
fn inc(mut c: C):
    c.n += 10
struct Node:
    var c: C
    var kids: List<Int>
struct Pool:
    _xs: List<Node>
    pub fn at(mut self, i: Int) -> lends mut Node:
        lend mut _xs[i]
    pub fn get(self, i: Int) -> Int = _xs[i].c.n
var p = Pool([Node(C(1), [2]), Node(C(7), [])])
p[0].c.bump(4)
p[1].c.n = 2
p[1].c.n += 3
inc(mut p[0].c)
p[0].kids.push(3)
p[0].kids[0] += 1
p[1].kids.push(9)
p[0].kids[1] += 4
p[0].kids.swap(0, 1)
p[1].kids.clear()
print(p[0], p[1], p[0] + p[1], p.at(0).kids[0], p.at(1).kids.len)
