// expect: ok, output "4 1 2\n9"
// rule: R12.4.5
use std.seq.{Indexed}
fn first<S: Indexed<Int, T>, T>(s: S) -> T = s[0]
let xs = [[1, 2], [3]]
let r = xs[1]
print(first([4, 5]), r.len, xs[0][1])
var ys = [7, 8]
ys[1] += 1
print(ys[1])
