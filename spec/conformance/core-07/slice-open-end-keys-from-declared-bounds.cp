// expect: ok, output "0 0 7 0"
// rule: 07.4.5
// An open slice end takes the source's declared bound, so a literal inside it is a proven key (decision 388).
var c: [1..=4]Int = 0
c[2..][3] = 7
print(c)
