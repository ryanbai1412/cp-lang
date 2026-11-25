// expect: ok, output "3 2 1"
// input: "3\n1 2 3\n"
let n: 1..=100 = read()
let a: [0..n]Int = read()
var b: List<Int> = []
for i in 0..n:  // expect: push_loop
    b.push(a[n - 1 - i])
print(b)
