// cp: 0.1 1f59ac0+dirty
// AtCoder DP K Stones: two players alternately remove a[i] stones from a pile
// of k; whoever cannot move loses. A pile is winning iff some move leaves a
// losing pile.
let n: 1..=100 = read()
let k: 1..=1e5 = read()
domain Pile = 0..=k
let moves: [n]1..=1e5 = read()

var wins: [Pile]Bool = false
for p in Pile:
    for a in moves:
        if let rest = p - a and not wins[rest]: wins[p] = true
print(if wins[Pile.last]: "First" else: "Second")
