// IOI 2000 Post Office: n villages on a line, m post offices to build in
// villages. Minimise the sum over villages of the distance to the nearest
// office; print that sum and the office positions in increasing order.
// Split the sorted villages into m runs, each served from its median.
// original: corpus/submissions/4811886.cpp
let n: 1..=300 = read()
let m: 1..=30 = read()
let spots: [n]1..=10000 = read()
let at = sorted(spots)

// villages i..=j walking to one office at their median
@memo
fn serve(i: 0..n, j: 0..n) -> Int:
    sum(abs(at[t] - at[(i + j) / 2]) for t in i..=j)

// villages i.. with k offices left; none when villages remain and offices do not
@memo
fn best(i: 0..=n, k: 0..=m) -> ?Int:
    if i == n: return 0
    if k == 0: return none
    var least: ?Int = none
    for j in i..n:
        if let rest = best(j + 1, k - 1): least min= serve(i, j) + rest
    least

print(best(0, m)!)
var offices: List<Int> = []
var i = 0
var k = m
while i < n:
    var j = i
    while best(j + 1, k - 1) == none or serve(i, j) + best(j + 1, k - 1)! != best(i, k)!: j += 1
    offices.push(at[(i + j) / 2])
    i = j + 1
    k -= 1
print(offices)
