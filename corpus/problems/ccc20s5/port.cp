// Source: CCC 2020 S5 (Josh's Double Bacon Deluxe), https://dmoj.ca/problem/ccc20s5
// Problem: N <= 1e6 people queue, person i's favourite burger type is b_i <= 5e5, and the burgers
//   are exactly their favourites. The first takes a random burger; each later one takes their
//   favourite if left, else a random one. Print the probability the last person (Josh) gets his.
// Solution: a random pick of type x hurts only the last person with favourite x, who then picks
//   at random from what is left. dp[i] is Josh's chance when person i's burger is taken: 1 for
//   the first person's type, 0 for Josh's, (1 + sum of dp after i) / (n - i) if i is the last
//   of its type, else dp of that last one. Scan from the back with a suffix sum; the answer is
//   sum dp / n. O(N + M).
// original: corpus/submissions/5032745.cpp
// Bounds: n: 1..=1e6; fav: 1..=5e5
let n: 1..=1e6 = read()
domain Pos = 0..n
domain Kind = 1..=5e5
let fav: [Pos]Kind = read()
let coach = fav[Pos.first!]
let josh = fav[Pos.last!]

var last_of: [Kind]?Pos = None
for i in Pos: last_of[fav[i]] = i

// chance[i]: Josh's chance when the chain of random picks reaches person i's burger first.
var chance: [Pos]F64 = 0.0
var after = 0.0  // the sum of chance over the people from i on
for i in Pos.rev:
    let last = last_of[fav[i]]!
    chance[i] = if fav[i] == coach: 1.0
                elif fav[i] == josh: 0.0
                elif last == i: (1.0 + after) / F64(n - i.idx)
                else: chance[last]
    after += chance[i]
print(if coach == josh: 1.0 else: after / F64(n), precision: 8)
