// expect: ok, output "4 5 6"
use std.seq.{rekey}
domain K = 10..13
let b = rekey([4, 5, 6], K.range)
print(b[K(10)], b[K(11)], b[K(12)])
