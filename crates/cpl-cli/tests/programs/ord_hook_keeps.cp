// `max`, `min`, `max=` and `min=` on a struct with its own `Ord` follow its `lt`, not its fields.
struct P(x: Int)
impl Ord for P:
    fn lt(a, b) = b.x < a.x
impl Show for P:
    fn show(p) = "{p.x}"

let ps = [P(1), P(3), P(2)]
print(ps.max()!, ps.min()!, max(P(1), P(3)), ps.sorted())
var best = P(2)
for p in ps: best max= p
var worst = P(2)
for p in ps: worst min= p
var some: ?P = None
for p in ps: some max= p
print(best, worst, some!)
var pair = (P(2), 0)
pair max= (P(1), 5)
print(pair.0, pair.1)
