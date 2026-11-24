// Source: CCO 2018 P1 (Geese vs. Hawks), https://dmoj.ca/problem/cco18p1
// limits: 600 ms, 1024 MB
// Problem: each team played n <= 1000 games, given in order as W/L and points (<= 1e6). Pick
//   rivalry games: pairs (geese game i, hawks game j), in order in both lists, where one team
//   won and scored more than the other. Print the largest total points of both teams in them.
// Solution: an LCS-style DP: best[g, h] over the first g geese and h hawks games is the best of
//   skipping either last game or pairing the two when they can be rivals. O(n^2).
let n: 1..=1000 = read()
let (geese_won, geese): (Str, [0..n]1..=1e6) = read()
let (hawks_won, hawks): (Str, [0..n]1..=1e6) = read()

fn rivals(g: 0..n, h: 0..n) -> Bool =
    geese_won[g] != hawks_won[h] and (
        if geese_won[g] == 'W': geese[g] > hawks[h] else: geese[g] < hawks[h]
    )

// best[g, h]: the most points from the first g geese games and h hawks games
var best: [0..=n, 0..=n]Int = 0
for g in 0..n:
    for h in 0..n:
        let paired = if rivals(g, h): best[g, h] + geese[g] + hawks[h] else: 0
        best[g + 1, h + 1] = max(best[g, h + 1], best[g + 1, h], paired)
print(best[n, n])
