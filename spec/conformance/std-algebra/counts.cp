// expect: ok, output "YES\nYES YES 6"
use std.algebra.{MinCount, MaxCount, MaxSub}
print(MinCount.of(5).count == 1 and MinCount.join(MinCount.of(2), MinCount.of(2)).count == 2 and MinCount.join(MinCount(1, 3), MinCount(2, 5)).val == 1 and fold(MinCount.monoid(), [MinCount.of(v) for v in [3, 1, 1]]).count == 2 and MaxCount.join(MaxCount(4, 1), MaxCount(4, 2)).count == 3 and fold(MaxCount.monoid(), [MaxCount.of(v) for v in [3, 1, 3]]).count == 2)
print(MaxSub.join(MaxSub.of(-1), MaxSub.of(-2)).try_best == Some(-1), MaxSub.monoid().id.try_best == None, (MaxSub.of(3) * 2).try_best!)
