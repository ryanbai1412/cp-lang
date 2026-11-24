// Source: Codeforces 2261D2 (Come Be My Friend, Don't Be So Scared, Hard Version),
//   https://codeforces.com/contest/2261/problem/D2
// limits: 2000 ms, 256 MB
// Problem: n spots in a row and m distinct detector segments [l, r], each reporting whether the
//   hidden spot is inside it. Add the fewest segments so the reports always identify the spot;
//   print the count and the segments. n, m <= 2e5 (sums over all tests). Multitest.
// Solution: XOR a random 62-bit key per segment over its range: equal hashes mean equal covering
//   sets, and each equal-hash pair needs a new boundary between them. Greedy stabbing from the
//   left (carry the nearest later partner, cut where it is next door) gives the fewest cuts.
//   Pair cut i with cut i + half as one segment, adding cut 0 when the outermost bad pair still
//   collides and cut n to make the count even. O((n + m) log n).
// Bounds: n: 1..=2e5; m: 0..=2e5; l, r: 1..=n

@!multitest
let (n, m): (1..=2e5, 0..=2e5) = read()
var diff: [0..=n]Int = 0
for _ in 0..m:
    let (l, r): (1..=n, 1..=n) = read()
    let key = rand(0, (1 << 62) - 1)
    diff[l - 1] ^= key
    diff[r] ^= key
let xors = scan(diff[0..n], ^)
let hash = [xors.prefix(i + 1) for i in 0..n]

var next: [0..n]?Int = None
var (first_bad, last_bad): (?Int, ?Int) = (None, None)
let order = (0..n).sorted(by: i => (hash[i], i))
for (a, b) in order.adjacent if hash[a] == hash[b]:
    next[a] min= b
    first_bad min= a
    last_bad max= b

var cuts: List<Int> = []
if let f = first_bad and let l = last_bad and hash[f] == hash[l]: cuts.push(0)
for i in 0..n - 1:
    if next[i] == i + 1: cuts.push(i + 1)
    else: next[i + 1] min= next[i]
if cuts.len % 2 == 1: cuts.push(n)

let half = cuts.len / 2
print(half)
for i in 0..half: print(cuts[i] + 1, cuts[i + half])
