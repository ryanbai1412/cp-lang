let n: 1..=5 = read()
domain Game = 1..=n
enum Outcome:
    @tag("W") Win
    @tag("L") Loss
type Season = [Game](outcome: Outcome, points: 1..=1e6)
// the input gives the season column by column: read each column, then zip them into rows
let outcomes: [Game]Outcome = read()
let points: [Game]1..=1e6 = read()
let geese: Season = [Game] g => (outcome: outcomes[g], points: points[g])
let grid: [Game, Game]Outcome = read()
print(sum(g.points for g in geese if g.outcome == Win), count(grid[i, i] == Loss for i in Game))
