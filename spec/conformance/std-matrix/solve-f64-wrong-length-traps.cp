// expect: R0401 (checked)
print(yes(Mat<F64>.from_rows([[1.0, 0.0], [0.0, 1.0]]).solve([1.0], eps: 1e-9) == None)) // <- here
