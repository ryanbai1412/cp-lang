// expect: ok, output "0 10"
use std.timetree.{TimeTree}
print(TimeTree<Int>(2).run(mark: () => 0, apply: e => (), answer: t => t * 10, undo: m => ()))
