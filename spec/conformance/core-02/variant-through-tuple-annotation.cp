// expect: ok, output "1 1"
// rule: 02.7.1
// The expected type reaches bare variants through a tuple annotation and `if` arms (rebuild round 16 H-VARIANT-TUPLE).
enum Side = Left | Right
let k = 3
let (cost, side): (Int, Side) = if k > 2: (1, Left) else: (2, Right)
print(cost, if side == Left: 1 else: 0)
