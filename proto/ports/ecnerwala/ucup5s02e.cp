// Universal Cup 5 Stage 2 (EPS) E: a cut after position i is good when two of
// the three rankings have the same set of contestants in their first i+1
// places; the answer is the number of good cuts plus one.
@!multitest
let n: 1..=2e5 = read()
domain P = 1..=n
let rank: [0..3][0..n]P = read()
var good: [0..n - 1]Bool = false
for z in 0..3:
    for y in z + 1..3:
        var place: [P]Int = 0
        for i in 0..n: place[rank[y][i]] = i
        var reach = 0  // furthest place in y of anyone in z's prefix
        for i in 0..n - 1:
            reach max= place[rank[z][i]]
            if reach == i: good[i] = true
print(1 + count(good))
