// expect: ok, output "26"
// A method that gives a part of its receiver on one return and builds a value on another:
// the part is read in place where the caller only reads it, and the built value moves.
struct Bag:
    id: List<Int>
    xs: List<Int>
    fn pick(self, k: Int) -> lends List<Int>:
        if k == 0: return self.id
        self.xs + [k]
let b = Bag(id: [7, 8], xs: [1])
var s = 0
for k in 0..3:
    for x in b.pick(k): s += x
    let p = b.pick(k)
    s += p.len
print(s)
