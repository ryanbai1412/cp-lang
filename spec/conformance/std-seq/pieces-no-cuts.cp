// expect: ok, output "1 10"
use std.seq.{pieces}
let cuts: List<Int> = []
let (domain P, piece) = (0..10).pieces(cuts: cuts)
print(P.len, piece[P.all.start].len)
