// expect: E0302 at 7
// rule: 01.8.7
// has: "a.inv()"
struct Z(v: Int)
impl Mul for Z:
    fn mul(self, o: Z) -> Z = Z(self.v * o.v)
impl Z:
    fn inv(self) -> Z = Z(self.v)
let a = Z(3)
print((a * a.inv).v)
