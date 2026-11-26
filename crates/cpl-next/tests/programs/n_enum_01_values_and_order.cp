// expect: ok, output "-1 0 1 true false true 2 true"
// rule: R02.7.3, R08.1.6
// A variant's written value is its `Int`; variants that carry nothing compare in declaration order.
enum Sign: Neg = -1, Zero = 0, Pos = 1
enum Dir = N | E | S
fn sign(x: Int) -> Sign = if x > 0: Sign.Pos elif x < 0: Sign.Neg else: Sign.Zero
print(Int(sign(-5)), Int(sign(0)), Int(sign(7)), sign(-2) == Sign.Neg, sign(1) == Sign.Neg, Sign.Neg < Sign.Pos, Int(Dir.S), Dir.E != Dir.S)
