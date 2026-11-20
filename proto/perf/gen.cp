// cp: 0.1 5df8551
let n: 1..=1e7 = read()
fn evens(m: 1..=1e7) -> yields Int:
    var i = 0
    while i < m:
        yield i
        i += 2
var reps = 0
var s = 0
while reps < 20:
    for v in evens(n):
        s += v % 7
    reps += 1
print(s)
