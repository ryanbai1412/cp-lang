// expect: ok, output "-3 Neg"
// rule: 08.1.5
// An enum variant named like an operator trait (`Sign.Neg`) doesn't hide the trait in `impl Neg`.
enum Sign = Neg | Pos
struct P(x: Int)
impl Neg for P:
    fn neg(a) = P(-a.x)
impl Show for Sign:
    fn show(self) = if self == Sign.Neg: "Neg" else: "Pos"
print((-P(3)).x, Sign.Neg)
