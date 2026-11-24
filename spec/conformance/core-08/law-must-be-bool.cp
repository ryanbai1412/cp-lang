// expect: E0302
// rule: 08.3.1
trait Mix:
    fn mix(self, a: Int, b: Int) -> Int
    law comm(a: Int, b: Int) = self.mix(a, b) // <- here
