// expect: ok, output "-9 10"
// engines: next
// The old pipeline's std (pinned at 6a31a3747^) predates `trait Dir`, which a2cba6caf added.
use std.grid.{Dir, Dir8}
fn step_of<D: Dir>(d: D) -> Int:
    let (r, c) = d.delta
    r * 10 + c
print(step_of(Dir8.all()[1]), step_of(Dir8.all()[4]))
