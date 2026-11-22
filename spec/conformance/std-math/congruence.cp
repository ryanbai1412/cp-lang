// expect: ok, output "YES"
use std.math.{linear_diophantine, linear_congruence}
print(linear_diophantine(240, 46, 4) == Some((5, -26, 23, 120)) and linear_diophantine(3, 0, 6) == Some((2, 0, 0, 1)) and linear_diophantine(2, 4, 3) == None and linear_congruence(6, 4, 10) == Some((4, 5)) and linear_congruence(2, 1, 4) == None and linear_congruence(0, 0, 3) == Some((0, 1)))
