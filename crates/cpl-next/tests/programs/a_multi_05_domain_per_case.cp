// expect: ok, output "3\n1"
// rule: R10.1.1
// input: "2\n3\n1\n"
// A domain declared below the divider is fresh per case; its size is the case's.
@!multitest
let n: Int = read()
domain V = 0..n
var a: [V]Int = 0
var c = 0
for v in V:
    a[v] = 1
    c += a[v]
print(c)
