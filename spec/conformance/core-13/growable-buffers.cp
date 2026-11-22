// expect: ok, output "100000 3"
// rule: 13.2.10
var xs: List<Int> = []
for i in 0..100000: xs.push(i)
var s = "ab"
s.push('c')
print(xs.len, s.len)
