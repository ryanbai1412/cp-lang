// expect: ok, output "1\n2\n1\n5"
// rule: 10.1.1
// input: "2\n3\n4\n"
// A name declared below `@!multitest` may hold the case's positions; one above may hold what is not a position.
var total = 0
@!multitest
let n: 1..=5 = read()
domain V = 0..n
var keep = []
keep.push(V(0))
total += V.last!.ord
print(keep.len)
print(total)
