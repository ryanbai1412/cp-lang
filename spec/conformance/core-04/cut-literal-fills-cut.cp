// expect: ok, output "0 2"
// rule: 04.2.18
// An integer literal where a V.cuts is expected is that cut.
domain V = 0..5
var mex: V.cuts = 0
let two: V.cuts = 2
print(Int(mex), Int(two))
