// expect: ok, output "0 1 0 1"
// rule: R08.1.2, R08.1.5
struct Rev<T>: x: T
impl<T: Eq> Eq for Rev<T>:
    fn eq(a, b) = a.x == b.x
impl<T: Ord> Ord for Rev<T>:
    fn lt(a, b) = b.x < a.x
fn b(c: Bool) -> Int = if c: 1 else: 0
print(b(Rev(1) < Rev(2)), b(Rev(2) < Rev(1)), b(Rev(Rev(2)) < Rev(Rev(1))), b(Rev(3) == Rev(3)))
