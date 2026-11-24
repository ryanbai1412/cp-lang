// expect: L0608
// rule: 07.1.7
// A method that does work still reads as a call: `z.inv` is not a field.
struct Z(v: Int)
impl Z:
    fn inv(self) -> Int = self.v
let z = Z(3)
print(z.inv + 1) // <- here
