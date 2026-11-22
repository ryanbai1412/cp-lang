// expect: ok, output "YES"
use std.math.{min_mod_in_range, continued_fraction}
print(min_mod_in_range(3, 7, 5, 6) == Some(2) and min_mod_in_range(2, 4, 1, 1) == None and min_mod_in_range(5, 9, 0, 3) == Some(0) and continued_fraction(415, 93) == [4, 2, 6, 7] and continued_fraction(-7, 3) == [-3, 1, 2] and continued_fraction(0, 5) == [0])
