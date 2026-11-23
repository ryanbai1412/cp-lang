// expect: ok, output "4 0"
// rule: 04.4.1
domain Seg = 0..2
domain P = 0..4
var piece: [Seg]Range<P.cuts> = P.cuts.first!..P.cuts.first!
piece[Seg(0)] = P.cuts.first!..P.cuts.last!
print(piece[Seg(0)].len, piece[Seg(1)].len)
