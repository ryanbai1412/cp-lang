// Source: IOI 2000 (Post Office), https://dmoj.ca/problem/ioi00p5
// Problem: V <= 300 villages at distinct positions <= 1e4 on a line. Build P <= 30 post
//   offices in villages, minimising the sum of each village's distance to its nearest office.
//   Print the sum, then the office positions in increasing order.
// Solution: the offices split the sorted villages into P runs, each best served from its
//   median. best(i, k) = least cost for villages i.. with k offices = min over the first run
//   i..=j of serve(i, j) + best(j + 1, k - 1), both memoised; then follow the optimal choices
//   to print the medians. O(V^2 P), plus O(V^3) for serve.
// original: corpus/submissions/4811886.cpp
// Bounds: n: 1..=300; m: 1..=30; spots: 1..=10000
let n: 1..=300 = read()
let m: 1..=30 = read()
let spots: [n]1..=10000 = read()
let at = spots.sorted()

// villages i..=j walking to one office at their median
@memo
fn serve(i: 0..n, j: 0..n) -> Int:
    sum(abs(at[t] - at[(i + j) / 2]) for t in i..=j)

// villages i.. with k offices left; none when villages remain and offices do not
@memo
fn best(i: 0..=n, k: 0..=m) -> ?Int:
    if i == n: return 0
    if k == 0: return None
    var least: ?Int = None
    for j in i..n:
        if let rest = best(j + 1, k - 1): least min= serve(i, j) + rest
    least

print(best(0, m)!)
var offices: List<Int> = []
var i = 0
var k = m
while i < n:
    var j = i
    while best(j + 1, k - 1) == None or serve(i, j) + best(j + 1, k - 1)! != best(i, k)!: j += 1
    offices.push(at[(i + j) / 2])
    i = j + 1
    k -= 1
print(offices)
