// expect: ok, output "5 0"
use std.dp.{backtrack, Step}
for s in backtrack(5, x => if x < 0: [x] else: []):
    let st: Step<Int> = s
    print(st.state, st.depth)
