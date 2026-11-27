// expect: ok, output "3\nYES\n3 YES\n9"
// rule: R03.1.4
// A method's type parameters that its receiver names are the receiver's, not made by the
// method: `top` on `Bx<1..=3>` is that type's own and gives `1..=3`'s `MAX`, and `pop` on a
// `List<0..=9>` is that list's own, beside the `Int` ones.
struct Bx<T>:
    xs: List<T>
impl<T: Bounded> Bx<T>:
    fn top(self) -> ?T = Some(T.MAX)
let b = Bx<1..=3>([])
let c = Bx<Int>([])
if let t = b.top(): print(t)
if let u = c.top(): print(yes(u > 3))
var xs: List<0..=9> = [2, 3]
var ys: List<Int> = [9]
if let i = xs.pop(): print(i, yes(xs.len == 1))
if let j = ys.pop(): print(j)
