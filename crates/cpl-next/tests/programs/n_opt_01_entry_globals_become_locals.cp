// expect: ok, output "2404"
// rule: R02.10.2
// input: "4\n"
// Optimized IR makes a scalar global that only the top level names one of its locals, and
// forwards a temporary into the one statement reading it; `n`, which `scaled` reads, stays
// a global. The results are the reference interpreter's.
let n: 1..=10 = read()
fn scaled(x: Int) -> Int = x * n
var total = 0
var count = 0
for i in 0..n:
    total += scaled(i)
    count += 1
print(total * 100 + count)
