// expect: ok, output "1"
// rule: 08.2.8
trait Zero:
    fn zero() -> Self
struct Cnt: c: Int
impl Zero for Cnt:
    fn zero() -> Cnt = Cnt(0)
fn make<T: Zero>() -> T = T.zero()
let x: Cnt = make()
print(x.c + 1)
