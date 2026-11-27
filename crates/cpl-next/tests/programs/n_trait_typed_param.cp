// expect: ok, output "30 12\n7"
// rule: R08.2.1
// A parameter typed by a trait is a type parameter of its own bounded by the trait, in a free
// fn and a method alike.
trait Big<V>:
    fn big(self) -> V
struct A(x: Int)
struct B(s: Str)
impl Big<Int> for A:
    fn big(self) -> Int = self.x * 10
impl Big<Int> for B:
    fn big(self) -> Int = self.s.len * 4
fn show(b: Big<Int>) -> Int = b.big()
struct Sum(n: Int):
    pub fn of(a: Big<Int>, b: Big<Int>) -> Sum = Sum(a.big() / 10 + b.big() / 4)
print(show(A(3)), show(B("abc")))
print(Sum.of(A(4), B("xyz")).n)
