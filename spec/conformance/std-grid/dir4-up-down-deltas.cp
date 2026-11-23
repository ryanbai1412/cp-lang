// expect: ok, output "-1 0 1 0 YES"
use std.grid.{Dir4}
let u = Dir4.up.delta
let d = Dir4.down.delta
print(u.0, u.1, d.0, d.1, yes(Dir4.up.back == Dir4.down))
