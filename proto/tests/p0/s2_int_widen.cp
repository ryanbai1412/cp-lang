// Synthesis #2 rule 4: an Int next to an F64 widens in + - * / **; Int / Int stays integer division.
let a: Int = read()
let b: Int = read()
let h = 0.5
let total = a * b / F64(b) + h                     // one cast marks the expression as floating
print("{total:.2f} {a / b} {a / F64(b):.3f} {F64(a) ** b:.1f} {2.0 ** a:.1f} {a + h:.1f} {h - a:.1f} {h * b:.1f}")
var acc = 0.25
acc += a
acc *= b
acc -= 1
acc /= b
print("{acc:.4f}")
fn through(j: Int, p: Int) -> F64 = h + F64(j) ** p
print("{through(3, 2):.1f}")
