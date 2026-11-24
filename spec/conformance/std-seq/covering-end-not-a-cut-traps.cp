// expect: R0401 (checked)
use std.seq.{covering, pieces}
let (domain P, piece) = (0..10).pieces(cuts: [3, 7])
print(piece.covering(3..5).len) // <- here
