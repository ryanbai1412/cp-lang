// expect: E0606
// rule: R08.1.2
// `impl Tr for _..=_` is for every range type, and not for `Int`.
trait Ends:
    fn tag() -> Int
impl Ends for _..=_:
    fn tag() -> Int = 7
struct S<K: Ends>:
    x: Int
    fn get(self) -> Int = K.tag() + x
let a: S<0..=5> = S(x: 1)
print(a.get())
let b: S<Int> = S(x: 1)
print(b.get()) // <- here
