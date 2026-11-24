// expect: R0401
use std.seq.{rekey}
domain K = 10..13
let b = rekey([4, 5], K.range) // <- here
print(b[K(10)])
