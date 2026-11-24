// expect: E0202
// rule: 08.1.9
trait Pick:
    fn pick(self) -> Int
impl<S: Seq<Int>> Pick for S:
    fn pick(self) -> Int = 1
impl<S: Sized> Pick for S: // <- here
    fn pick(self) -> Int = 2
print([1].pick())
