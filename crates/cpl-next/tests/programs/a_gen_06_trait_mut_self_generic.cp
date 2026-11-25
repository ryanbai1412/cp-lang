// expect: ok, output "4"
// rule: R08
// trait impl chosen per instance, method with a mut self called through a generic fn with a mut param.
trait Grow:
    fn grow(mut self)
struct A:
    var n: Int
impl Grow for A:
    fn grow(mut self):
        n += 1
fn twice<T: Grow>(mut x: T):
    x.grow()
    x.grow()
var a = A(2)
twice(mut a)
print(a.n)
