// expect: ok, output "2"
// A value typed by a trait has that trait's type: `s` walks `from: Seq<V>` as a `V`, here a product cell.
use std.seq.{Seq}
domain R = 0..2
domain C = 0..2
domain Cell = (R, C)
fn unseen<V>(from: Seq<V>, seen: [V]?Int) -> Int:
    var c = 0
    for s in from:
        if seen[s] == None:
            c += 1
    c
var seen: [Cell]?Int = None
seen[(R(1), C(0))] = Some(1)
print(unseen([(R(0), C(0)), (R(1), C(0)), (R(1), C(1))], seen))
