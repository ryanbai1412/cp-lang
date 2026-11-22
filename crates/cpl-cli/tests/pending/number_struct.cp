// Syntax batch 1: an `Int` is made into any number struct (one with a
// `zero`, a `one` and a `new(x: Int)`), a type gives the numbers it was
// built with, and a struct `let` may be capitalized.
struct Ring<P>(v: Int):
    let MOD = P
    let zero = Ring(0)
    let one = Ring(1 % P)
    fn new(x: Int) -> Ring<P> = Ring(x % P)
impl Add for Ring<P>:
    fn add(a, b) = Ring<P>.new(a.v + b.v)
impl Mul for Ring<P>:
    fn mul(a, b) = Ring<P>.new(a.v * b.v)
type R = Ring<7>
print(R.MOD, Ring<11>.MOD)
var x: R = 5
x += 4
x *= 3
print(x.v)
if x == 6: print("six")
var xs: List<R> = [1, 2, 3]
(xs[0], xs[2]) = (xs[2], xs[0])
print(xs[0].v, xs[..2].len)
struct Box<T>(v: T, w: T)
let b = Box(1, 2)
let c = Box("a", "b")
print(b.v, c.w)
struct Bag<D, T>(a: [D]T):
    fn first(self) -> T = self.a[D.first]
let n = 3
domain I = 0..n
let arr: [I]Int = [I] i => i.to_idx + 7
let bag = Bag(arr)
print(bag.first)
