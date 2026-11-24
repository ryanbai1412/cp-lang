// Source: IOI 2013, Robots, https://dmoj.ca/problem/ioi13p5
// Problem: T <= 1e6 toys with weight and size; A weak robots carry toys of weight strictly below
//   their limit, B small robots toys of size strictly below theirs (A, B <= 5e4, values <= 2e9).
//   Each robot puts away one toy per minute. Print the fewest minutes for all toys, or -1.
// Solution: binary search the minutes m. Check: weak robots by rising limit each take the m
//   largest-size toys among those light enough (a heap of sizes, toys sorted by weight); then
//   small robots by falling limit each take the m largest left, failing when one is too big.
//   O(T log^2 T).
// Bounds: a, b: 0..=5e4; t: 1..=1e6
// original: corpus/submissions/4688418.cpp (a grader-style `putaway`; orig.cpp adds a main)
type Limit = 1..=2e9
let (a, b, t): (0..=5e4, 0..=5e4, 1..=1e6) = read()
domain Weak = 0..a
domain Small = 0..b
domain Toy = 0..t
var weak: [Weak]Limit = read()
var small: [Small]Limit = read()
let toys: [Toy](weight: Limit, size: Limit) = read()

weak.sort()
small.sort(by: desc)
// The sizes of the toys each weak robot (by rising limit) is the first one
// strong enough for, and the sizes of the toys no weak robot can carry.
var newly: [Weak]List<Desc<Int>> = []
var heavy: List<Desc<Int>> = []
for toy in toys:
    if let r = weak.bsearch(w => w > toy.weight).right: newly[r].push(desc(toy.size))
    else: heavy.push(desc(toy.size))

fn done_in(minutes: Int) -> Bool:
    var left = MinHeap<Desc<Int>>()  // largest size first
    for r in Weak:
        left.push_all(newly[r])
        for _ in 0..min(minutes, left.len): left.pop()
    left.push_all(heavy)
    for limit in small:
        for s in left.drain().take(minutes):
            if s.x >= limit: return false
    left.len == 0

domain Minutes = 1..=t
if let m = Minutes.first_true(m => done_in(m.label)): print(m)
else: print(-1)
