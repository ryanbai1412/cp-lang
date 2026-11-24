// Source: CCO 2018 P1 (Geese vs. Hawks), https://dmoj.ca/problem/cco18p1
// Problem: each team played n <= 1000 games, given in order as W/L and points (<= 1e6). Pick
//   rivalry games: pairs (geese game i, hawks game j), in order in both lists, where one team
//   won and scored more than the other. Print the largest total points of both teams in them.
// Solution: an LCS-style DP: best[g, h] over the first g geese and h hawks games is the best of
//   skipping either last game or pairing the two when they can be rivals. O(n^2).
// Bounds: n: 1..=1000; geese, hawks: 1..=1e6

let n: 1..=1000 = read()
domain Game = 0..n
let (geese_won, geese): (Str, [Game]1..=1e6) = read()
let (hawks_won, hawks): (Str, [Game]1..=1e6) = read()

// One team won, and the winner scored strictly more.
fn rivals(g: Game, h: Game) -> Bool =
    geese_won[g.ord] != hawks_won[h.ord] and
        (if geese_won[g.ord] == 'W': geese[g] > hawks[h] else: geese[g] < hawks[h])

// best[g, h]: the most points from the first g geese games and h hawks games
var best: [Game.cuts, Game.cuts]Int = 0
for g in Game:
    for h in Game:
        let paired = if rivals(g, h): best[g.l, h.l] + geese[g] + hawks[h] else: 0
        best[g.r, h.r] = max(best[g.l, h.r], best[g.r, h.l], paired)
print(best[Game.cuts.last!, Game.cuts.last!])
