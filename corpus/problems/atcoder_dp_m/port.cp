// Educational DP M, Candies: share exactly k candies among n children, child i
// getting at most a[i]. Count the ways.
// original: corpus/submissions/4131778.cpp
const P = 1000000007
type Z = Mod<P>
let n: 1..=100 = read()
let k: 0..=1e5 = read()
domain Given = 0..=k
let a: [n]0..=k = read()

// ways[g]: ways to give out g candies among the children so far. The next
// child takes up to `most` of them, so it sums a window of the row before.
var ways: [Given]Z = 0
ways[Given.first] = 1
for most in a:
    var next: [Given]Z = 0
    var window: Z = 0  // ways[g - most ..= g]
    for g in Given:
        window += ways[g]
        if let gone = g - (most + 1): window -= ways[gone]
        next[g] = window
    ways = next
print(ways[Given.last])
