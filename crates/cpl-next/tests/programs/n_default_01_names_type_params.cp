// expect: ok, output "0 0\n7\n16 7\n5"
// A default is read where its fn or struct is declared, with the declaration's type parameters
// naming this call's types, so `id: F = F.zero` is `Int.zero` at an `F = Int` call; and a named
// type's self-less trait fn is reached as `Int.zero` (R06.1.5, R08.2.8).
use std.algebra
trait Seed:
    @get fn seed() -> Self
impl Seed for Int:
    @get fn seed() -> Int = 7
struct Cell<F: Seed>(v: F)
impl<F: Seed> Cell<F>:
    fn make(v: F = F.seed) -> Cell<F> = Cell(v)
fn first<F: Seed>(x: F = F.seed) -> F = x
print(Int.zero, U64.zero)
print(Cell<Int>.make().v)
let a = Action<Int, Int>(apply: (d, x, span) => x + d * span)
print(a.apply(2, 10, 3), a.compose(2, 5))
print(first<Int>(5))
