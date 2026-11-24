// expect: ok, output "2"
// rule: 08.1.9
// A trait's default fn is checked with the trait's own associated-type default: `Self.Tk` is `Lz<Self>` in `tk`'s body (decision 469).
struct Lz<S>(s: S, k: Int)
trait Sq:
    type Tk = Lz<Self>
    fn tk(self, k: Int) -> Self.Tk = Lz(self, k)
struct W(v: Int)
impl Sq for W
print(W(3).tk(2).k)
