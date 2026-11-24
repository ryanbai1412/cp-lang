let n: 1..=5 = read()
domain Game = 1..=n
enum Outcome:
    @tag("W") Win
    @tag("L") Loss
type Season = [Game](outcome: Outcome, points: 1..=1e6)
let geese: Season = read(columns)
let grid: [Game, Game]Outcome = read()
print(sum(g.points for g in geese if g.outcome == Win), count(grid[i, i] == Loss for i in Game))
