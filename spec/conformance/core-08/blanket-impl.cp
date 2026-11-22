// expect: ok, output "7 0"
// rule: 08.1.2
trait Total:
    fn total(self) -> Int
impl<T: Total> Total for ?T:
    fn total(self) -> Int:
        if let x = self:
            return x.total()
        return 0
struct C: c: Int
impl Total for C:
    fn total(self) -> Int = self.c
let a: ?C = C(7)
let b: ?C = None
print(a.total(), b.total())
