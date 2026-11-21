// Hungarian on a dense 1500 x 1500 matrix with weights up to 10^9.
use std.flow.{min_weight_matching}

domain I = 0..1500
var seed = 1
fn next(n: Int) -> Int:
    seed = seed * 48271 % 2147483647
    seed % n

let w = [I] _ => [I] _ => next(1000000000)
print(min_weight_matching::<I, I>((i, j) => w[i][j]).total)
