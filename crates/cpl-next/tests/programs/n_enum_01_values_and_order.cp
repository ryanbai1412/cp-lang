// expect: ok, output "0 1 2 YES NO YES 2 YES"
// rule: R02.7.3, R08.1.6
// A payload-free variant's `Int` is its position, and variants compare in declaration order.
enum Sign = Neg | Zero | Pos
enum Dir = N | E | S
fn sign(x: Int) -> Sign = if x > 0: Sign.Pos elif x < 0: Sign.Neg else: Sign.Zero
print(Int(sign(-5)), Int(sign(0)), Int(sign(7)), yes(sign(-2) == Sign.Neg), yes(sign(1) == Sign.Neg), yes(Sign.Neg < Sign.Pos), Int(Dir.S), yes(Dir.E != Dir.S))
