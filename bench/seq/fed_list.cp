// cf2237i2's shape: a fn returning a small List, walked by a `for` that runs to
// its end, 100,000,000 calls (the loop runs at each push, no list per call).
fn step(x: Int) -> List<Int>:
    var out: List<Int> = []
    out.push(x)
    if x % 3 == 0:
        out.push(-x)
    out
let n = 100000000
var s = 0
for i in 0..n:
    for v in step(i):
        s += v
print(s)
