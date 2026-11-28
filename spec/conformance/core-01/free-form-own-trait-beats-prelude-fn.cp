// expect: ok, output "3"
// rule: 01.10.1
// engines: next
// The free name has its trait's level: the module's own trait gives `tally` over the prelude's `tally`, as an own fn of the name would.
trait Tally:
    @free_form
    fn tally(self) -> Int
struct P(n: Int)
impl Tally for P:
    fn tally(self) -> Int = self.n
print(tally(P(3)))
