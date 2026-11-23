// CCC 2020 S5 Josh's Double Bacon Deluxe: n customers queue for burgers; the
// first takes a random burger, and everyone else takes their favorite if left
// or a random remaining one. Find the probability the last person gets theirs.
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
