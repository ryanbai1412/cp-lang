use std.dp.aliens
// The most a contestant gets from exactly k disjoint nonempty subarrays:
// that total is concave in k, so pay λ per subarray and let `aliens` find λ.
let (n, k): (1..=1e6, 1..=1e6) = read()
let a: [0..n]-1e6..=1e6 = read()
fn better(x: (val: Int, count: Int), y: (val: Int, count: Int)) -> Bool =
    x.val < y.val or x.val == y.val and x.count < y.count
fn solve(lam: Int) -> (val: Int, count: Int):
    var out = (val: 0, count: 0)
    var open = (val: 0, count: 0)
    var started = false
    for x in a:
        let fresh = (val: out.val - x + lam, count: out.count + 1)
        open = if started and better((val: open.val - x, count: open.count), fresh): (val: open.val - x, count: open.count) else: fresh
        started = true
        if better(open, out): out = open
    out
print(-aliens(k, 0..=2 * 10 ** 6, solve))
