// expect: ok, output "2 YES YES 10 30\n3 4 3"
// rule: 08.1.1
use std.seq.{pieces}
let (domain C, xs) = compress([30, 10, 30])
print(xs.len, yes(xs.contains(30)), yes(not xs.contains(20)), xs[xs.at(10)], xs[xs.at(30)])
let (domain P, piece) = (0..10).pieces(cuts: [3, 7])
print([piece[p].len for p in P])
