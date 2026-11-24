// expect: ok, output "1 1 1 1 2"
// rule: 08.1.6
// R08.1.6: a struct's own `Eq` and `Hash` key a `HashMap` and a `HashSet` too, and so std's
// `distinct` and `tally`, which keep their keys in them.
struct H(a: Int, b: Int)
impl Eq for H:
    fn eq(self, o: H) -> Bool = self.a == o.a
impl Hash for H:
    fn hash(self) -> U64 = self.a.hash()
var m: HashMap<H, Int> = HashMap()
m[H(1, 2)] += 1
m[H(1, 3)] += 1
let s = HashSet([H(1, 2), H(1, 3)])
let t = [H(1, 2), H(1, 5)].tally()
print(m.len, s.len, [H(1, 2), H(1, 5)].distinct().len, t.len, t[H(1, 9)])
