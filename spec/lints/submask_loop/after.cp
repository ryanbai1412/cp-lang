use std.math.{submasks}
let n: 1..=16 = read()
let w: [0..(1 << n)]Int = read()
var best = [0..(1 << n)] _ => 0
for mask in 0..(1 << n):
    for s in submasks(mask):
        if s > 0:
            best[mask] max= w[s]
print(best)
