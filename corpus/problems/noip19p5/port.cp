// NOIP 2019 Day 2 P2, Partition: cut a[1..=n] into consecutive blocks
// whose sums never decrease, minimising the sum of the squared block
// sums. Print the minimum.
//
// The best cut makes the last block as short as it can be: for a prefix
// i, take the latest j whose own last block fits, s[j] - s[from[j]] <=
// s[i] - s[j]. The threshold 2 s[j] - s[from[j]] is what a queue keeps
// increasing, and the latest j under s[i] only moves right.
//
// n reaches 4e7, so the large tests give a generator instead of the
// array, and the answer reaches 1.6e33, past `Int`: it is summed in `I128`.
// input: `n 0` and the array, or `n 1`, `x y z b1 b2 m` and m lines
// `p l r`: b[i] = (x b[i-1] + y b[i-2] + z) mod 2^30, and a[i] =
// b[i] mod (r - l + 1) + l for the first line with i <= p.
// original: corpus/submissions/5148191.cpp
let (n, kind): (Int, Int) = read()
domain Pos = 1..=n
var a: [Pos]Int = [Pos] i => 0
if kind == 0:
    for i in Pos:
        let v: Int = read()
        a[i] = v
else:
    let (x, y, z): (Int, Int, Int) = read()
    var (older, old): (Int, Int) = read()  // b[i - 2], b[i - 1]
    let m: Int = read()
    var i = 1
    for _ in 0..m:
        let (p, l, r): (Int, Int, Int) = read()
        while i <= p:
            var b = if i == 1: older else: old
            if i > 2:
                b = (x * old + y * older + z) & ((1 << 30) - 1)
                (older, old) = (old, b)
            a[Pos(i)] = b % (r - l + 1) + l
            i += 1

// s[c]: sum of the values before cut c; from[c]: where the last block
// of the best partition of those values starts.
type Cut = Cut<Pos>
var s: [Cut]Int = [Pos.cuts] c => 0
for i in Pos: s[i.r] = s[i.l] + a[i]
var from: [Cut]Cut = [Pos.cuts] c => Pos.cuts.first!
fn threshold(j: Cut) -> Int = 2 * s[j] - s[from[j]]

var queue: List<Cut> = [Pos.cuts.first!]
var head = 0
for i in Pos:
    let c = i.r
    while head + 1 < queue.len and threshold(queue[head + 1]) <= s[c]: head += 1
    from[c] = queue[head]
    while queue.len > head and threshold(queue[queue.len - 1]) >= threshold(c): queue.pop()
    queue.push(c)

var total: I128 = 0
var c = Pos.cuts.last!
while c != Pos.cuts.first!:
    total += I128(s[c] - s[from[c]]) ** 2
    c = from[c]
print(total)
