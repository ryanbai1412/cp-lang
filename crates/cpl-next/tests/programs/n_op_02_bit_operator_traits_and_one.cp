// expect: ok, output "6 5 7 2\n12 7 10"
// rule: R08.1.5
// `^`, `|` and `&` are core's operator traits as `+` is, so a struct of your own takes them and a
// bound on a type parameter allows them; `T.one` is where a product starts, as `T.zero` is a sum's.
struct B(x: Int)
impl BitXor for B:
    fn bitxor(self, o: B) -> B = B(self.x ^ o.x)
fn p<T: Mul + One>(a: T, b: T) -> T = T.one * a * b
fn x<T: BitXor + BitOr + BitAnd>(a: T, b: T) -> T = (a ^ b) | (a & b)
print((B(5) ^ B(3)).x, 6 ^ 3, 6 | 1, 6 & 3)
print(p(3, 4), x(6, 3), p(U64(2), U64(5)))
