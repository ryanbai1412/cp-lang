// expect: ok, output "12 1024 27"
// rule: 08.1.5
// `**` is std's `Pow` trait: a user impl takes it, and generic code over `Pow` takes the numbers.
struct P(v: Int)
impl Pow for P:
    fn pow(self, e: Int) -> P = P(self.v * e)
fn cube<T: Pow>(x: T) -> T = x ** 3
print((P(3) ** 4).v, 2 ** 10, cube(3))
