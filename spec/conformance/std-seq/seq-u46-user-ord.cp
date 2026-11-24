// expect: ok, output "1 3 1 3"
// Acceptance for seq/U46 (sort by a user order), handbook/product/std/sequences-usecases.md: `sort`, `sorted`, `max` and `min` all follow `impl Ord`'s `cmp`.
struct P:
    x: Int
impl Ord for P:
    fn cmp(self, o: P) -> Ordering = o.x.cmp(self.x)
var ps = [P(1), P(3), P(2)]
let top = ps.sorted()[0].x
ps.sort()
print(ps.max()!.x, top, ps.last!.x, ps.min()!.x)
