// expect: ok, output "1 2 0"
// rule: R08.1.2, R08.1.5
struct P: x: Int
impl Eq for P:
    fn eq(a, b) = a.x == b.x
impl Ord for P:
    fn lt(a, b) = a.x < b.x
fn order<T: Ord>(a: T, b: T) -> Int = if a < b: 1 elif b < a: 2 else: 0
print(order(P(1), P(3)), order(7, 3), order(P(2), P(2)))
