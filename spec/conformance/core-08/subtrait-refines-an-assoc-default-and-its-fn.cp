// expect: ok, output "2 102"
// rule: 08.1.9
// A subtrait refines its supertrait's associated-type default and the default fn using it; a type implementing both takes the most derived one, with no ambiguity (decision 469).
struct Lz<S>(s: S, k: Int)
struct Sl<S>(s: S, k: Int)
trait Sq<A>:
    type Tk = Lz<Self>
    fn tk(self, k: Int) -> Self.Tk = Lz(self, k)
trait Ix<A>: Sq<A>:
    type Tk = Sl<Self>
    fn tk(self, k: Int) -> Self.Tk = Sl(self, k + 100)
struct W(v: Int)
impl Sq<Int> for W
struct X(v: Int)
impl Sq<Int> for X
impl Ix<Int> for X
print(W(3).tk(2).k, X(3).tk(2).k)
