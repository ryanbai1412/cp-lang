// expect: ok, output "5"
// rule: 08.3.1
trait Mix:
    fn mix(self, a: Int, b: Int) -> Int
    law comm(a: Int, b: Int) = self.mix(a, b) == self.mix(b, a)
struct Plus: z: Int
impl Mix for Plus:
    fn mix(self, a: Int, b: Int) -> Int = a + b
print(Plus(0).mix(2, 3))
