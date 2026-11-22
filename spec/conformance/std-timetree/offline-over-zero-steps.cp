// expect: ok, output "0"
use std.timetree.{offline_over}
let r = offline_over(0, [], by: e => (0, 0), mark: () => 0, apply: e => (), answer: t => t, undo: m => ())
print(r.len)
