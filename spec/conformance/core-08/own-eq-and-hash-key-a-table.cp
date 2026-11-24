// expect: ok, output "1 1 YES YES"
// rule: 08.1.6
// R08.1.6: a struct's own `Eq` and `Hash` replace the structural ones everywhere, as a table's
// key and inside a tuple or a list.
struct Half(n: Int)
impl Eq for Half:
    fn eq(a, b) = a.n / 2 == b.n / 2
impl Hash for Half:
    fn hash(self) -> U64 = (self.n / 2).hash()
let s = Set([Half(2), Half(3)])
var m: Map<Half, Int> = Map()
m[Half(4)] += 1
m[Half(5)] += 1
print(s.len, m.len, yes((Half(2), 1) == (Half(3), 1)), yes([Half(2)] == [Half(3)]))
