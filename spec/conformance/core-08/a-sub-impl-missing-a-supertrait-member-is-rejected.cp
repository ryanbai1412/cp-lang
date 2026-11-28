// expect: E0606
// rule: 08.1.1
// An impl serving its supertrait writes each of the supertrait's members that has no default.
trait Shape:
    fn area(self) -> Int
    fn sides(self) -> Int
trait Big(Shape):
    fn big(self) -> Bool = self.area() > 10
struct Sq(s: Int)
impl Big for Sq: // <- here
    fn area(self) -> Int = self.s * self.s
