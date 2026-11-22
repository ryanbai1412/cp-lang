// Codeforces 2147 D — Game on Array
//
// Each move decrements all copies of a chosen positive value `x` by 1 and
// scores its frequency `freq[x]`. Every even height `2 * (x / 2)` is split
// equally (`x / 2` turns for each player) because the second player can always
// mirror the first player's move on that value, giving `base = sum(x / 2)` to
// both players.
//
// Each distinct odd value leaves one extra turn worth `freq[x]` points; optimal
// play claims these odd-value multiplicities in descending order, alternating
// between player 0 and player 1 (`score[k % 2] += c`).

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]1..=10 ** 9 = read()

var freq: Map<Int, Int> = Map()
for x in a: freq[x] += 1

let base = sum(x / 2 for x in a)
let extras = [cnt for (x, cnt) in freq.items if x % 2 == 1].sorted(by: c => -c)
var score: [0..2]Int = base
for (k, c) in extras.items: score[k % 2] += c
print(score[0], score[1])
