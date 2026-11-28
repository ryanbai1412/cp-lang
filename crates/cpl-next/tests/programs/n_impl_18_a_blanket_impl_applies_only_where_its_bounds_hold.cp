// expect: ok, output "3 7"
// rule: R08.1.2
// A blanket impl applies only where its bounds hold: `Int` has no `to` field, so the row-bounded impl is skipped and the plain blanket one applies; a record with `to` takes the row-bounded one, the higher rank (R08.1.9). std's `Target<V>` impls are this shape.
trait Small:
    fn small(self) -> Int
impl Small for Int:
    fn small(self) -> Int = self
trait Dest:
    fn dest(self) -> Int
impl<R: (to: Int, ..)> Dest for R:
    fn dest(self) -> Int = self.to
impl<V: Small> Dest for V:
    fn dest(self) -> Int = self.small()
fn via<T: Dest>(x: T) -> Int = x.dest()
print(via(3), via((to: 7, w: 1)))
