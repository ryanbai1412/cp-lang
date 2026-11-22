// expect: ok, output "YES"
use std.math.{mult_order, jacobi, carmichael}
print(yes(mult_order(2, 7) == Some(3) and mult_order(3, 7) == Some(6) and mult_order(2, 4) == None and mult_order(5, 1) == Some(1) and jacobi(2, 7) == 1 and jacobi(3, 7) == -1 and jacobi(7, 7) == 0 and jacobi(2, 15) == 1 and carmichael(1) == 1 and carmichael(8) == 2 and carmichael(15) == 4 and carmichael(7) == 6))
