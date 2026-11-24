// expect: ok, output "YES"
print(yes(abs(Mat<F64>.from_rows([[1.0, 2.0], [2.0, 4.0]]).det(eps: 1e-9)) < 1e-9))
