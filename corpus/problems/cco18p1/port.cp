// CCO 2018 P1 Geese vs. Hawks: each team played n games. Pair up games, in
// order, into rivalries: one team won its game, the other lost, and the winner
// scored more. Print the largest total of points over the paired games.
let n: 1..=1000 = read()
let (geese_won, geese): (Str, [0..n]1..=1e6) = read()
let (hawks_won, hawks): (Str, [0..n]1..=1e6) = read()

fn rivals(g: 0..n, h: 0..n) -> Bool =
    geese_won[g] != hawks_won[h] and (if geese_won[g] == 'W': geese[g] > hawks[h] else: geese[g] < hawks[h])

// best[g, h]: the most points from the first g geese games and h hawks games
var best: [0..=n, 0..=n]Int = 0
for g in 0..n:
    for h in 0..n:
        let paired = if rivals(g, h): best[g, h] + geese[g] + hawks[h] else: 0
        best[g + 1, h + 1] = max(best[g, h + 1], best[g + 1, h], paired)
print(best[n, n])
