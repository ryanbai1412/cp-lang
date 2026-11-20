// A program may declare its own `struct SegTree` or `struct Dsu`: the name is then the program's, not the
// built-in type. A record with a `?Int` or a `List<?Int>` field builds.
struct SegTree:
    sz: Int
    v: List<?Int>
    fn update(mut self, pos: Int, x: Int):
        self.v[pos] = x
struct Dsu:
    p: List<Int>
    fn find(self, x: Int) -> Int:
        if self.p[x] == x: x else: self.find(self.p[x])
var t = SegTree(2, [none, none])
t.update(1, 5)
let d = Dsu([0, 0])
print(t.v[1] ?? -1, d.find(1))
