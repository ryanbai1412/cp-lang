// Forms a trace has to show: a table fill, a `mut` place alias, a `lend`
// from a method, a trait `law`, a row `without`, a `has` bound and an
// `unchecked:` block.
trait Mix:
    fn mix(self, a: Int, b: Int) -> Int
    law comm(a: Int, b: Int) = self.mix(a, b) == self.mix(b, a)
struct Plus: z: Int
impl Mix for Plus:
    fn mix(self, a: Int, b: Int) -> Int = a + b

struct Cells: t: List<Int>
impl Cells:
    fn at(mut self, i: Int) -> lends mut Int:
        lend mut t[i]

fn strip<R: (u: Int, ..)>(r: R) -> (..R without (u)) = (..r without (u))

fn kind<T>(x: T) -> Str:
    if T: Mix: "mix" else: "plain"

let n = 3
var a: [0..n]Int = [0..n]0
let p = mut a[1]
p += 10
var c = Cells([1, 2, 3])
c.at(2) += 5
let s = strip((u: 1, w: 7))
print(a[1], c.t[2], s.w, Plus(0).mix(2, 3), kind(Plus(0)), kind(1))
unchecked:
    print(a[1] * 2)
