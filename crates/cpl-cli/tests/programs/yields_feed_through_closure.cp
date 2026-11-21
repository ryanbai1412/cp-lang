// A loop over a `yields` fn whose body calls a fn value still runs the
// body as each value is made, since no closure here writes what the fn reads.
var seen = 0
fn upto(n: Int) -> yields Int:
    for i in 0..n:
        seen += 1
        yield i * seen

fn total(n: Int, w: Int -> Int) -> Int:
    var t = 0
    for x in upto(n):
        t += w(x)
        if t > 1000: break
    t

let k: Int = read()
print(total(4, x => x * k), total(100, x => x + k), seen)
