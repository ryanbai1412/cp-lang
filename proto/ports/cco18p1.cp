// cp: 0.1 c217120+dirty
// CCO 2018 P1 Geese vs. Hawks: each team played n games. Pair up games, in
// order, into rivalry games: one team won its game, the other lost, and the
// winner scored more. Print the largest total of points over paired games.
let n: 1..=1000 = read()
domain Game = 1..=n

@tags("W", "L")
enum Outcome:
    Win
    Loss

type Season = [Game](outcome: Outcome, points: 1..=1e6)
let geese: Season = read(columns)
let hawks: Season = read(columns)

fn rivalry(g: Game, h: Game) -> Bool:
    let (a, b) = (geese[g], hawks[h])
    a.outcome != b.outcome and (if a.outcome == Win: a.points > b.points else: a.points < b.points)

// most points from the geese games after bound x and the hawks games after y
@memo
fn best(x: Game.cuts, y: Game.cuts) -> Int:
    let (g, h) = (x.right, y.right) else: return 0
    let paired = if rivalry(g, h): geese[g].points + hawks[h].points + best(g.r, h.r) else: 0
    max(best(g.r, y), best(x, h.r), paired)
print(best(Game.first.l, Game.first.l))
