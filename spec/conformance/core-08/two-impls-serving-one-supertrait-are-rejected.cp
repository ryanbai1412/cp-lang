// expect: E0202
// rule: 08.1.1
// Two impls that each write a supertrait's members are two impls of it for one type.
trait Shape:
    fn area(self) -> Int
trait Big(Shape):
    fn big(self) -> Bool = self.area() > 10
trait Small(Shape):
    fn small(self) -> Bool = self.area() < 5
struct Sq(s: Int)
impl Big for Sq:
    fn area(self) -> Int = self.s * self.s
impl Small for Sq: // <- here
    fn area(self) -> Int = self.s
