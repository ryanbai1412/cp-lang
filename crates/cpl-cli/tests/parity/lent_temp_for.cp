// A `for` walks a view made over temporaries: they live through the whole loop.
use std.seq.{zip}
fn go(n: Int) -> Int:
    var s = 0
    for (x, y) in zip([i * 3 for i in 0..n], [i + 1 for i in 0..n]): s += x * y
    for (k, (x, y)) in zip([i for i in 0..n][1..], [5 for _ in 0..n]).items: s += k * x + y
    s
let n: Int = read()
for (x, y) in zip([i for i in 0..n], [n - i for i in 0..n]): print(x, y)
print(go(n), go(2 * n))
