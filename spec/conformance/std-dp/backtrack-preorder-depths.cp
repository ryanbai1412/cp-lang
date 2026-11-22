// expect: ok, output "YES\n1 2 4 5 3 6 7\n1"
use std.dp.{backtrack}
print(backtrack(1, x => if x < 4: [2 * x, 2 * x + 1] else: []).map(s => s.depth).collect() == [0, 1, 2, 2, 1, 2, 2])
print([s.state for s in backtrack(1, x => if x < 4: [2 * x, 2 * x + 1] else: [])])
print(backtrack(0, x => if x < 0: [x] else: []).collect().len)
