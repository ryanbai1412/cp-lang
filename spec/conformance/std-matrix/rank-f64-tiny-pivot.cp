// expect: ok, output "1"
print(Mat<F64>.from_rows([[1.0, 0.0], [0.0, 1e-12]]).rank(eps: 1e-9))
