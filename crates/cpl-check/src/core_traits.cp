// The core traits the compiler gives meaning to and std has no source for,
// as their fns' declared types (R08.1.5). An impl fn that writes no type
// for a parameter or its result takes these, with `Self` the impl's target
// and a missing trait argument `Self`: `fn add(a, b)` in `impl Add for P`
// takes two `P`, and `fn mul(a, k)` in `impl Mul<Int> for P` a `P` and an
// `Int`. The checker and the lowering both read this file.
trait Add<Rhs>:
    fn add(a: Self, b: Rhs) -> Self
trait Sub<Rhs>:
    fn sub(a: Self, b: Rhs) -> Self
trait Mul<Rhs>:
    fn mul(a: Self, b: Rhs) -> Self
trait Div<Rhs>:
    fn div(a: Self, b: Rhs) -> Self
trait Rem<Rhs>:
    fn rem(a: Self, b: Rhs) -> Self
trait BitAnd<Rhs>:
    fn bitand(a: Self, b: Rhs) -> Self
trait BitOr<Rhs>:
    fn bitor(a: Self, b: Rhs) -> Self
trait BitXor<Rhs>:
    fn bitxor(a: Self, b: Rhs) -> Self
trait Neg:
    fn neg(a: Self) -> Self
trait Eq<Rhs>:
    fn eq(a: Self, b: Rhs) -> Bool
trait Ord<Rhs>:
    fn lt(a: Self, b: Rhs) -> Bool
    fn cmp(a: Self, b: Rhs) -> Ordering
trait Show:
    fn show(self) -> Str
