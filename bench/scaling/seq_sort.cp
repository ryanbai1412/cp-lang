// fixed: i
let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
var s = 0
for r in 0..1024:
    var a = xs.clone() // timed: seq.clone
    a.dedup() // timed: seq.dedup
    a.reverse() // timed: seq.reverse
    a.rotate_left(n / 3) // timed: seq.rotate_left
    a.rotate_right(n / 3) // timed: seq.rotate_right
    s += a.resized(n + 1, 0).len // timed: seq.resized
    a.insert(0, 5)
    s += a.remove(0)
print(s)
