// expect: ok, output "0"
use std.timetree.{TimeTree}
let r = TimeTree<Int>(0).run(mark: () => 0, apply: e => (), answer: t => t * 10, undo: m => ())
print(r.len)
