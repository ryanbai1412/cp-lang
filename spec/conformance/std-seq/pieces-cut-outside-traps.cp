// expect: R0401 (checked)
use std.seq.{pieces}
let (domain P, piece) = (0..10).pieces(cuts: [11]) // <- here
print(P.len)
