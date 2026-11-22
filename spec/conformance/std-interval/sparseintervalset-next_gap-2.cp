// expect: ok, output "YES"
use std.interval.{SparseIntervalSet}
print(yes(SparseIntervalSet().next_gap(-7) == -7))
