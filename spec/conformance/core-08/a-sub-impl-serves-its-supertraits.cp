// expect: ok, output "16 16\nbig"
// rule: 08.1.1
// An impl that writes its supertrait's members is also the type's impl of that supertrait.
trait Shape:
    fn area(self) -> Int
trait Big(Shape):
    fn big(self) -> Bool = self.area() > 10
struct Sq(s: Int)
impl Big for Sq:
    fn area(self) -> Int = self.s * self.s
fn area_of<T: Shape>(t: T) -> Int = t.area()
let q = Sq(4)
print(q.area(), area_of(q))
if q.big(): print("big")
