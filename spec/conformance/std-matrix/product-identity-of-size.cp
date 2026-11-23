// expect: ok, output "YES"
let m = Mat<Int>.product(2)
let a = Mat<Int>.from_rows([[1, 2], [3, 4]])
print(yes(m.op(m.id, a).get(1, 0) == 3))
