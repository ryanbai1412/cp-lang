// A struct's `impl Ord` given by the spec's `cmp` orders it everywhere: comparisons, sorts, min/max, keeps and tuples.
struct P(x: Int)
impl Ord for P:
    fn cmp(a, b) = cmp(b.x, a.x)
impl Show for P:
    fn show(p) = "{p.x}"

let ps = [P(1), P(3), P(2)]
print(yes(P(1) < P(3)), yes(P(1) > P(3)), yes(P(2) <= P(2)), yes(P(1) >= P(3)))
print(ps.sorted(), ps.max()!, ps.min()!, max(P(1), P(3)))
var best = P(2)
for p in ps: best max= p
print(best, yes((P(1), 0) < (P(3), 0)), [(P(1), 1), (P(3), 2)].sorted())
