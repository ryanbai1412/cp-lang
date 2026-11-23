// expect: ok, output "YES"
use std.math.{best_rational}
print(yes(Ratio(2, 4) == Ratio(1, 2) and Ratio(3, -6).floor == -1 and Ratio(5) == Ratio.from_int(5) and Ratio.from_int(3) == Ratio(3, 1) and Ratio(-2, 3).inv == Ratio(-3, 2) and Ratio(1, 4).inv == Ratio.from_int(4) and Ratio(7, 2).floor == 3 and Ratio(-7, 2).floor == -4 and Ratio(7, 2).ceil == 4 and Ratio(-7, 2).ceil == -3 and Ratio(-1, 2).abs == Ratio(1, 2)))
