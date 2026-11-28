// expect: ok, output "3.0000000000 0.5000000000 5.0000000000 -0.2500000000 1200.0000000000"
// rule: 10.2.6
// input: "+3 .5 5. -.25 12E+2\n"
// An `F64` token may lead with `+`, and its `.` may have digits on one side only.
let (a, b, c, d, e): (F64, F64, F64, F64, F64) = read()
print(a, b, c, d, e)
