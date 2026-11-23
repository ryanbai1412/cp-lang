// cf2001D Longest Max Min Subsequence: take every distinct value once. The next pick lies
// after the previous one and no later than the earliest last occurrence of a value still
// unused; odd steps take the largest candidate, even steps the smallest, leftmost on ties.
@!multitest
let n: 1..=3e5 = read()
domain P = 1..=n
let a = read<[P]1..=n>()
var last: [1..=n]?P = None
for i in P: last[a[i]] = Some(i)
var deadline = SortedSet<P>()
for x in 1..=n: if let p = last[x]: deadline.add(p)

var used: [1..=n]Bool = false
var after = 0                         // position of the previous pick, 0 before the first
var hi: Heap<(Int, P)> = Heap()          // (-value, position)
var lo: Heap<(Int, P)> = Heap()
var next = 1                          // next position to enter the candidates
var b = List<Int>()
while let r = deadline.first:
    while next <= Int(r):
        hi.push((-a[P(next)], P(next)))
        lo.push((a[P(next)], P(next)))
        next += 1
    let fresh = (x: Int, p: P) => not used[x] and Int(p) > after
    var pick: ?(Int, P) = None
    if b.len % 2 == 0:
        while let (neg, at) = hi.pop():
            if fresh(-neg, at):
                pick = Some((-neg, at))
                break
    else:
        while let e = lo.pop():
            if fresh(e.0, e.1):
                pick = Some(e)
                break
    let (x, p) = pick!
    b.push(x)
    used[x] = true
    after = Int(p)
    deadline.remove(last[x]!)
print(b.len)
print(b)
