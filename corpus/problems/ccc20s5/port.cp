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
let n: 1..=1000000 = read()
domain Pos = 0..n
let fav: [Pos]1..=500000 = read()

let coach = fav[Pos.first!]
let josh = fav[Pos.last!]
if coach == josh:
    print(1.0, precision: 8)  // WISH P4
else:
    var last_of: [1..=500000]?Pos = None
    for i in Pos: last_of[fav[i]] = i

    // dp[i]: probability Josh gets his burger if the chain of forced steals
    // first reaches someone at or after i whose favorite is fav[i].
    var dp: [Pos]F64 = 0.0     // WISH P4
    var suffix_sum: F64 = 0.0  // WISH P4
    for i in Pos.rev:
        if fav[i] == coach:
            dp[i] = 1.0
        elif fav[i] == josh:
            dp[i] = 0.0
        elif last_of[fav[i]] == i:
            let rem = n - i.idx
            dp[i] = (1.0 + suffix_sum) / F64(rem)  // WISH P4
        else:
            dp[i] = dp[last_of[fav[i]]!]
        suffix_sum += dp[i]

    print(suffix_sum / F64(n), precision: 8)  // WISH P4
