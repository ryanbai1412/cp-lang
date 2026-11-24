// expect: R0401 (checked)
print(Mat<Int>.from_rows([[1, 2], [3, 4]]).mul_vec([1]).len) // <- here
