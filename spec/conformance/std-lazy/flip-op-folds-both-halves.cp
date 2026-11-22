// expect: ok, output "YES\nYES\n4 9"
use std.lazy.{Flip}
print(Flip.op(+).op(Flip(1, 0), Flip(0, 1)).now == 1)
print(Flip.op(+).id.flipped == 0)
let m = Flip<Int>.op(max)
let z = m.op(m.op(Flip.of(true, 4, 0), Flip.of(false, 9, 0)), m.id)
print(z.now, z.flipped)
