// expect: ok, output "YES\nYES YES 6"
use std.algebra.{MinCount, MaxCount, MaxSub}
print(yes(MinCount.of(5).count == 1 and MinCount.join(MinCount.of(2), MinCount.of(2)).count == 2 and MinCount.join(MinCount(1, 3), MinCount(2, 5)).val == 1 and [MinCount.of(v) for v in [3, 1, 1]].fold(MinCount.monoid()).count == 2 and MaxCount.join(MaxCount(4, 1), MaxCount(4, 2)).count == 3 and [MaxCount.of(v) for v in [3, 1, 3]].fold(MaxCount.monoid()).count == 2))
print(yes(MaxSub.join(MaxSub.of(-1), MaxSub.of(-2)).try_best == Some(-1)), yes(MaxSub.monoid().id.try_best == None), (MaxSub.of(3) * 2).try_best!)
