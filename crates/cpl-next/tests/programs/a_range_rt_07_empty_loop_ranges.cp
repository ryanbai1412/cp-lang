// expect: ok, output "0"
// rule: spec 4: a range whose start is past its end is empty
// input: "0\n"
// Spec 4: a range whose start is past its end is empty: for over 5..n with n=0 runs zero times.
let n: Int = read()
var c = 0
for i in 5..n: c += 1
for i in 5..=n: c += 1
print(c)
