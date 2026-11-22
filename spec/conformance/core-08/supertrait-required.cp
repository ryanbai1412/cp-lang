// expect: E0606 at 6
// rule: 08.1.1
trait Shape:
    fn area(self) -> Int
trait Big(Shape):
    fn big(self) -> Bool = self.area() > 10
struct Sq: s: Int
impl Big for Sq
