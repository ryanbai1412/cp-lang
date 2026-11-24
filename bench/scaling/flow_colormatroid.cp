// n: 2048
// ColorMatroid copies `color` and sizes a [E]Bool. At n = 2048 both blocks stay under 128 KB at
// 4n too; at n = 32768 the 4n blocks outgrow 128 KB and the time grew 13x, not 4x.
use std.flow.{ColorMatroid}
let n: Int = read()
domain E = 0..n
domain C = 0..4
let col = [E] e => C(e.ord % 4)
let lim = [C] _ => n
var s = 0
for _ in 0..65536:
    let cm = ColorMatroid(col, lim) // timed: flow.ColorMatroid.new
    if cm.can_add(E(0)): s += 1
print(s)
