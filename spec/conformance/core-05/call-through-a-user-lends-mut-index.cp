// expect: ok, output "5 3 7 5 2"
// rule: 05.6.3
// `p[k].f.m()` with `m` a `mut self` method changes the slot a struct's `lends mut` `at` lends, even beside a `get`.
struct C:
    var n: Int
    fn bump(mut self, by: Int):
        n += by
struct Node:
    var c: C
    var kids: List<Int>
struct Pool:
    _xs: List<Node>
    pub fn at(mut self, i: Int) -> lends mut Node:
        lend mut _xs[i]
    pub fn get(self, i: Int) -> Node = _xs[i]
var p = Pool([Node(C(1), [2])])
p[0].c.bump(4)
p[0].kids.push(3)
p[0].kids[0] += 5
p[0].kids.swap(0, 1)
print(p.at(0).c.n, p.at(0).kids[0], p.at(0).kids[1], p.get(0).c.n, p.at(0).kids.len)
