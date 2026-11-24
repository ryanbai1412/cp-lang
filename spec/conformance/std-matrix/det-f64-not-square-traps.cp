// expect: R0401 (checked)
print(yes(Mat<F64>.from_rows([[1.0, 2.0]]).det(eps: 1e-9) == 0.0)) // <- here
