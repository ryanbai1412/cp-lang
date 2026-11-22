// expect: ok, output "YES"
use std.timetree.{TimeStep, TimeTree}
print(TimeTree<Int>(2).walk().collect() == [TimeStep.Query(0), TimeStep.Query(1)])
