// Source: IOI 2017 P4 (The Big Prize), https://dmoj.ca/problem/ioi17p4
// Problem: interactive. n <= 2e5 boxes hold prizes of types 1..v, one diamond (type 1),
//   and more than k^2 of type t when type t - 1 has k. ask(i) returns how many boxes left and
//   right of i hold a more valuable prize. Find the diamond, ideally in <= 5000 asks.
// Solution: lollipops are the vast majority, so asking the first 450 boxes meets one and the
//   largest left + right seen is the count of non-lollipops. Divide and conquer on [l, r]
//   knowing the non-lollipops left of each end: walk left from mid past non-lollipops to a
//   lollipop c, then recurse on [l, c - 1] and [mid + 1, r] while their counts differ; a
//   sum of 0 is the diamond. About sqrt(n) log n asks.
// Stdin: n, then the hidden prize kinds (0 the diamond; a smaller kind is rarer).
// original: corpus/submissions/5659145.cpp
let n: 3..=200_000 = read()
domain Box = 0..n
domain Kind = 0..n
let kind: [Box]Kind = read()

// the judge: ask(i) is how many boxes left and right of i hold a rarer prize
var seen = Fenwick<Kind, Int>()
var rarer_left: [Box]Int = 0
for i in Box:
    rarer_left[i] = seen.prefix(kind[i].l)
    seen.add(kind[i], 1)
let judge = [Box] i => (l: rarer_left[i], r: seen.prefix(kind[i].l) - rarer_left[i])

// the contestant: each box is asked at most once; every lollipop sees the most rarer boxes
var asked: [Box]Bool = false
var widest = 0
fn ask(i: Box) -> (l: Int, r: Int):
    if not asked[i]:
        asked[i] = true
        widest max= judge[i].l + judge[i].r
    judge[i]

// the diamond within cuts a..b, knowing how many non-lollipops lie before a and before b:
// walk left from the middle past non-lollipops to a lollipop, then split around it
fn search(a: Cut<Box>, b: Cut<Box>, before_a: Int, before_b: Int) -> ?Box:
    if a >= b or before_a == before_b: return None
    let mid = Box((Int(a) + Int(b) - 1) / 2)
    for c in (a.r!..=mid).rev:
        let (l, r) = ask(c)
        if l + r == 0: return c
        if l + r == widest:
            if let left = search(a, c.l, before_a, l): return left
            return search(mid.r, b, l + Int(mid) - Int(c), before_b)
    search(mid.r, b, before_a + Int(mid.r) - Int(a), before_b)

// lollipops are the vast majority: asking the first 450 boxes meets one
let probe = (Box.first!..=Box(min(n, 450) - 1)).find(i => ask(i).l + ask(i).r == 0)
print(probe ?? search(Box.cuts.first!, Box.cuts.last!, 0, widest)!)
