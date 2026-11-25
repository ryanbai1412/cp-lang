// expect: ok, output "6"
// rule: decision 16
// A generator that calls itself (decision 16), three deep.
fn walk(n: Int) -> yields Int:
    if n > 0:
        yield n
        for x in walk(n - 1): yield x
var s = 0
for v in walk(3): s += v
print(s)
