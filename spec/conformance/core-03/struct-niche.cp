// expect: ok, output "1 -5\n-1"
// rule: 03.2.2
struct Edge: to: Int, w: Int
var best: ?Edge = None
let span: ?(lo: Int, hi: Int) = (lo: -5, hi: 5)
print(if best is None: 1 else: 0, span!.lo)
best = Edge(3, -1)
print(best!.w)
