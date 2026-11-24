// expect: ok, output "5 6 2 9 1"
// rule: 08.1.7
// A tuple or record meets a trait by its shape, so the methods the trait gives a body are its own, as `if T: Trait` decides.
trait Twice:
    fn twice(self) -> (Self, Self) = (self, self)
trait Square:
    @get fn w(self) -> Int
    fn area(self) -> Int = self.w * self.w
fn has<T>(x: T) -> Int:
    if T: Twice:
        return 1
    0
let t = (5, 6).twice()
let r = (w: 2, h: 3).twice()
print(t.0.0, t.1.1, r.0.w, (w: 3).area(), has((1, 2)))
