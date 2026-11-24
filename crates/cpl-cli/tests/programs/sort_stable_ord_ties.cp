// Every sort is stable (spec/lib/ops-ranges.md): elements an `impl Ord`
// calls equal keep their order, in the C++ as in the interpreter.
struct Frac: p: Int, q: Int
impl Ord for Frac:
    fn cmp(self, o: Frac) -> Ordering = (self.p * o.q).cmp(o.p * self.q)
let xs = [Frac(k % 3 + 1, (k % 3 + 1) * (k % 2 + 2)) for k in 0..40]
let s = xs.sorted()
print([f.p for f in s])
var t = xs.clone()
t.sort()
print([f.q for f in t])
var u = xs
u.sort(desc: true)
print([f.p * 10 + f.q for f in u])
