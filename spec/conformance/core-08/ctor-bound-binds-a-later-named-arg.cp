// expect: ok, output "1 YES"
// rule: 08.1.2
// A `new`'s bound binds the struct's params from the impl it selects, even past defaulted params: `act: Neg()` after `fill:` gives `F = Bool` through `A: Acts<T, F>`, not `F`'s default.
pub trait Acts<T, F>:
    fn id(self) -> F
struct NoAct
impl<T, F> Acts<T, F> for NoAct:
    fn id(self) -> F = F.zero
struct Neg
impl<T> Acts<T, Bool> for Neg:
    fn id(self) -> Bool = false
struct Box<T, F = Int>(xs: List<T>, tag: F):
    pub fn new<M, A: Acts<T, F>>(op: M, xs: ?List<T> = None, fill: ?T = None, act: A = NoAct()) -> Box<T, F> =
        Box(xs ?? [], act.id())
    fn flip(self, f: F) -> F = f
let b = Box(0, [1, 2], act: Neg())
let c = Box(0, [1])
print(c.flip(1), yes(b.flip(true)))
