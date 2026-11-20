// Educational DP K, Stones: two players take turns removing a[i] stones from
// a pile of k, for any i. Whoever cannot move loses. Who wins?
// original: corpus/submissions/4131775.cpp
let n: 1..=100 = read()
let k: 1..=1e5 = read()
let a: [n]1..=k = read()

// wins[s]: the player to move with s stones left can force a win.
var wins: [0..=k]Bool = false
for s in 0..=k:
    wins[s] = any(s >= x and not wins[s - x] for x in a)
print(if wins[k]: "First" else: "Second")
