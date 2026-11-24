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
