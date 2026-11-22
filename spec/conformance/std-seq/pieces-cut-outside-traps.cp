// expect: R0401 at 3 (checked)
use std.seq.{pieces}
let (domain P, piece) = (0..10).pieces(cuts: [11])
print(P.len)
