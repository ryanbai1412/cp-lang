// expect: ok, output "5 4 7\n6 6 7\n5 5 7"
// rule: R09.1.3 R09.1.4
// engines: next
// A trait's fn that declares `uses` follows its caller when it is called through a bound,
// as a direct call does: each impl's copy, and the trait's default body, see the caller's value.
trait Shifted:
    fn shifted(self) -> Int uses io
    fn twice(self) -> Int uses io = self.shifted() + io.base
    fn plain(self) -> Int = 7
impl Shifted for Int:
    fn shifted(self) -> Int uses io = self + io.base
fn via<T: Shifted>(x: T) -> Int uses io = x.shifted()
fn deep<T: Shifted>(x: T) -> Int uses io = via(x)
fn line<T: Shifted>(x: T) -> (Int, Int, Int) uses io = (deep(x), x.twice() - 1, x.plain())
let (a, b, c) = line(5)
print(a, b, c)
with io(base: 1):
    let (d, e, f) = line(5)
    print(d, e, f)
    print(5.shifted() - 1, via(4), 7)
