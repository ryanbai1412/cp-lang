// expect: ok, output "YES"
use std.math.{Ratio, best_rational}
print(best_rational(31_416, 10_000, 10) == Ratio(22, 7) and best_rational(1, 2, 10) == Ratio(1, 2) and best_rational(1, 2, 1) == Ratio(0, 1) and best_rational(314_159_265, 100_000_000, 1000) == Ratio(355, 113))
