// expect: ok, output "3 7 0"
let m = Flip.op(+)
let x = m.op(Flip(3, 7), m.id)
print(x.now, x.flipped, m.id.now)
