// expect: ok, output "1 0 1 0 0 1\n1"
// rule: R08.1.5
struct P: x: Int, y: Int
impl Eq for P:
    fn eq(a, b) = a.x == b.x and a.y == b.y
impl Ord for P:
    fn lt(a, b) = a.x < b.x or (a.x == b.x and a.y < b.y)
fn b(c: Bool) -> Int = if c: 1 else: 0
let p = P(1, 2)
let q = P(1, 3)
print(b(p < q), b(p > q), b(p <= q), b(p >= q), b(p == q), b(p != q))
print(match p.cmp(q):
    Less => 1
    _ => 0)
