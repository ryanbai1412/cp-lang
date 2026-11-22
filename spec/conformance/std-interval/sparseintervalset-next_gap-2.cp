// expect: ok, output "YES"
use std.interval.{SparseIntervalSet}
print(SparseIntervalSet().next_gap(-7) == -7)
