// AtCoder DP K Stones: two players take turns removing one of the allowed
// amounts from a pile of k stones; whoever cannot move loses. A pile wins
// when some move leaves a losing pile.
let (n, k): (1..=100, 1..=1e5) = read()
let moves: [0..n]1..=1e5 = read()

var wins: [0..=k]Bool = false
for pile in 0..=k: wins[pile] = any(not wins[pile - a] for a in moves if a <= pile)
print(if wins[k]: "First" else: "Second")
