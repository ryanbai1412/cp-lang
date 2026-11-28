// expect: ok, output "-10 1"
// engines: next
// The old pipeline's std (pinned at 6a31a3747^) predates `trait Dir`, which a2cba6caf added.
use std.grid.{Dir, Dir4}
fn step_of<D: Dir>(d: D) -> Int:
    let (r, c) = d.delta
    r * 10 + c
print(step_of(Dir4.up), step_of(Dir4.right))
