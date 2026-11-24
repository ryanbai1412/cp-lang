// fixed: i
let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
let low = (x: Int) => x % 64 // within: seq.group
var s = 0
for r in 0..16:
    var b = xs.clone()
    b.sort() // timed: seq.sort
    s += xs.group(low).len // timed: seq.group
print(s)
