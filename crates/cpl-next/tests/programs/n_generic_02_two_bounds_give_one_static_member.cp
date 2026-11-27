// expect: E0701
// rule: R08.1.4
// `T.z()` that two bounds of `T` give is no choice: the call names its trait.
trait P:
    fn z() -> Int
trait Q:
    fn z() -> Int
impl P for Int:
    fn z() -> Int = 1
impl Q for Int:
    fn z() -> Int = 2
fn f<T: P + Q>() -> Int = T.z()  // <- here
print(f<Int>())
