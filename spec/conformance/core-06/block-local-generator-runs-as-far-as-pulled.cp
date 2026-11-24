// expect: ok, output "5030"
// rule: 06.3.3 07.1.3 07.2.10
// A block-local generator runs only as far as its loop takes: `nat` steps five times, not forever,
// and writes the caller's `seen`; a recursive one passes its captured `n` down.
fn count(n: Int) -> Int:
    var seen = 0
    fn nat() -> yields Int:
        var i = 0
        while true:
            seen += 1
            yield i
            i += 1
    fn walk(k: Int) -> yields Int:
        yield k
        if k > 0:
            for x in walk(k - 1): yield x + n
    for x in nat():
        if x == n: break
    var s = 0
    for x in walk(3): s += x
    seen * 1000 + s
print(count(4))
