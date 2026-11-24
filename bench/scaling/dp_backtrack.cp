use std.dp.{backtrack}
let n: Int = read()
var s = 0

fn kids(x: Int) -> yields Int:
    if x < n: // within: dp.backtrack
        yield 2 * x + 1 // within: dp.backtrack
        yield 2 * x + 2 // within: dp.backtrack
for _ in 0..40:
    s += backtrack(0, kids).count() // timed: dp.backtrack [let: visited = 2 n]
for _ in 0..1 << 15:
    for st in backtrack(0, kids):
        if st.depth == 1: st.prune() // timed: dp.Step.prune
print(s)
