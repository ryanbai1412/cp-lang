// expect: E0805 at 5
// rule: 10.1.1
// input: "1\n3\n"
var keep = []
@!multitest
let n: 1..=5 = read()
domain V = 0..n
keep.push(V(0))
print(keep.len)
