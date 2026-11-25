// expect: ok, output "36 9 6"
// input: "\n"
// The same shapes with no write: accepted, and nothing is copied.
var log: List<Int> = [1, 2, 3]
var other: List<Int> = []
fn rec(x: Int) -> Int:
    if other.len < 2000: other.push(x + log.len)
    x
var f: Int -> Int = x => x
f = rec
var s = 0
for v in log:
    for w in log: s += f(v) * w
print("{s} {other.len} {other[8]}")
