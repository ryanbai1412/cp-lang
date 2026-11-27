// expect: ok, output "2 3 | 1 9 | 2 5 4"
// rule: R06.1.5
// A fn item with a variadic tail named where a fn of fewer parameters is wanted gives its tail
// none, and a call through a name bound to it binds as a direct call does.
struct P: a: Int, b: Int
impl Ord for P:
    fn cmp(self, o: P) -> Ordering = (self.a, self.b).cmp((o.a, o.b))
fn ap<T>(g: (T, T) -> T, x: T, y: T) -> T = g(x, y)
let ps = [P(2, 1), P(1, 4), P(2, 3)]
let hi = ps.reduce(max)!
let lo = ap(min, 1, 9)
let f = min
let g = max
print(hi.a, hi.b, "|", lo, ap(max, 1, 9), "|", f(3, 2), g(1, 5, 4), f(7, 4))
