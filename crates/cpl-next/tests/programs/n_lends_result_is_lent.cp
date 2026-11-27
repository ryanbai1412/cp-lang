// expect: ok, output "1 1\n1\n2 1\n5 6\n5 6 7"
// rule: R05.6.1
// A fn that `lends` a param returns a view of it, so its result is lent in every caller, of a
// param or of a field, however many calls one statement holds: dropping the views leaves the
// lender's storage alone, and the view's type is the one the fn returns.
fn at(xs: List<Int>, u: Int) -> lends Slice<List<Int>> = xs[u..u + 1]
struct G(arcs: List<Int>, start: List<Int>):
    pub fn out(self, u: Int) -> lends Slice<List<Int>>:
        let (lo, hi) = (start[u], start[u + 1])
        arcs[lo..hi]
let xs = [5, 6, 7]
print(at(xs, 0).len, at(xs, 1).len)
print(at(xs, 2).len)
let g = G(arcs: [5, 6, 7], start: [0, 2, 3])
print(g.out(0).len, g.out(1).len)
let nx = g.out(0).to_list()
print(nx[0], nx[1])
print(xs)
