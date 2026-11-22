// expect: ok, output "1"
use std.dp.{backtrack}
for s in backtrack(1, x => if x < 4: [2 * x, 2 * x + 1] else: []):
    print(s.state)
    s.prune()
