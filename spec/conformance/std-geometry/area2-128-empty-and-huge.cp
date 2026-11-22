// expect: ok, output "0 YES"
use std.geometry.{area2_128, Pt}
let empty: List<Pt> = []
let big = 1_000_000_000_000
let sq = [Pt(0, 0), Pt(big, 0), Pt(big, big), Pt(0, big)]
print(area2_128(empty), yes(area2_128(sq) == I128(big) * I128(big) * I128(2)))
