// expect: ok
// rule: 13.3.3
// tool: build -o -
// lacks: "std::function"
fn walk(n: Int) -> yields Int:
    if n > 0:
        yield n
        for x in walk(n - 1): yield x
var s = 0
for x in walk(10):
    if x == 7: break
    s += x
print(s)
