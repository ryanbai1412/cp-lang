// expect: ok, output "0 1"
use std.seq.{runs_linked}
let a: List<Int> = []
print([r.len for r in a.runs_linked((x, y) => y == x + 1)].len, [r.len for r in [7].runs_linked((x, y) => y == x + 1)][0])
