// expect: ok, output "YES"
print(yes(Mat<F64>.from_rows([[1.0, 2.0], [2.0, 4.0]]).solve([1.0, 0.0], eps: 1e-9) == None))
