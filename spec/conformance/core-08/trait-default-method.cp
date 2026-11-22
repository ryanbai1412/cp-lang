// expect: ok, output "18"
// rule: 08.1.1
trait Shape:
    fn area(self) -> Int
    fn twice(self) -> Int = self.area() * 2
struct Sq: s: Int
impl Shape for Sq:
    fn area(self) -> Int = self.s * self.s
print(Sq(3).twice())
