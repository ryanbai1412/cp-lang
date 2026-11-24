// expect: ok, output "0 3"
// rule: 08.1.9
// `impl<D: Domain> Tr for D` covers every domain beside `impl Tr for Int`, and a generic body calls the trait's static fn through its bound.
trait Positions:
    fn count() -> Int
impl<D: Domain> Positions for D:
    fn count() -> Int = D.count
impl Positions for Int:
    fn count() -> Int = 0
struct Box<D: Positions>(n: Int):
    pub fn new(dyn: Bool = false) -> Box<D>:
        if dyn: return Box<D>(0)
        Box<D>(D.count())
domain V = 0..3
print(Box<Int>(dyn: true).n, Box<V>().n)
