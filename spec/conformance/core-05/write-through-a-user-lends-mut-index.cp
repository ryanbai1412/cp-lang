// expect: ok, output "20 7 8"
// rule: 05.6.3
// `p[k].f = v` on a struct whose `at` lends mut writes through `at`, even beside a `get`.
struct Node:
    var v: Int
    var kids: List<Int>
    var inner: (w: Int, z: Int)
struct Pool:
    _xs: List<Node>
    pub fn at(mut self, i: Int) -> lends mut Node:
        lend mut _xs[i]
    pub fn get(self, i: Int) -> Int = _xs[i].v
var p = Pool([Node(v: 1, kids: [2, 3], inner: (w: 4, z: 5))])
p.at(0).v = 9
p.at(0).kids[0] = 7
p.at(0).inner.w = 8
p[0].v += 11
print(p[0], p.at(0).kids[0], p.at(0).inner.w)
