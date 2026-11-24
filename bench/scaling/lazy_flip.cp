use std.lazy.{Flip, flip_action}
let n: Int = read()
let reps = 1 << 17
var s = 0

for i in 0..reps:
    s += Flip.of(i % 2 == 0, i, 0).now // timed: lazy.Flip.of
    s += Flip.op(+).op(Flip(i, 0), Flip(0, i)).now // timed: lazy.Flip.op
    s += flip_action(+).apply(i % 2 == 0, Flip(i, 1), 4).now // timed: lazy.flip_action
print(s)
