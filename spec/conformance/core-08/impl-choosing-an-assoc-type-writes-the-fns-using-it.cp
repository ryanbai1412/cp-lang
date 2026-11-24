// expect: E0606
// rule: 08.1.9
// An impl that chooses its own `type Tk` must write each default fn whose type names `Self.Tk`: that body was checked against the trait's `Tk` (decision 469).
struct Lz<S>(s: S, k: Int)
trait Sq:
    type Tk = Lz<Self>
    fn tk(self, k: Int) -> Self.Tk = Lz(self, k)
struct W(v: Int)
impl Sq for W:
    type Tk = Int  // <- here
print(W(3).v)
