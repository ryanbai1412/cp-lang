// expect: E0602
// input: "\n"
// The same, where the push is behind a function value. Today this is accepted and prints garbage in release.
var log: List<Int> = [1, 2, 3]
fn rec(x: Int) -> Int:
    if log.len < 2000: log.push(x)
    x
var f: Int -> Int = x => x
f = rec
var s = 0
for v in log: s += f(v)  // <- here
print("{s} {log.len}")
