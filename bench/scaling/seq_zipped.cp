// A zip's two halves are fields: reading either is O(1) at any n.
let n: Int = read()
let xs = [i for i in 0..n]
let ys = [2 * i for i in 0..n]
let z = zip(xs, ys)
var s = 0
for i in 0..64 * n:
    s += z.first[i % n] // timed: seq.Zipped.first
    s += z.second[(i + 1) % n] // timed: seq.Zipped.second
print(s)
