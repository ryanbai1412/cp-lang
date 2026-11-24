// expect: ok, output "12\n21"
// rule: 04.2.10
// A product position is viewed as its factor tuple, so it fits a `(R, C)` param and a method on `(R, C)` takes it.
domain R = 0..3
domain C = 0..3
domain Cell = (R, C)
fn code(at: (R, C)) -> Int = Int(at.0) * 10 + Int(at.1)
@get fn flip(self: (R, C)) -> Int = Int(self.1) * 10 + Int(self.0)
let p = Cell((R(1), C(2)))
print(code(p))
print(p.flip)
