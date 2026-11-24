// An enum whose tags are one letter each reads as a `Char`: a column of them
// reads packed (`WLW`) or spaced (`W L W`), as `[V]Char` does (R10.2.3).
enum Outcome:
    @tag("W") Win
    @tag("L") Loss
enum Step = @tag('U') Up(by: Int) | @tag('D') Down(by: Int)
let n: 1..=5 = read()
domain Game = 1..=n
let packed: [Game]Outcome = read()
let spaced: [Game]Outcome = read()
let grid: [Game, Game]Outcome = read()
let steps: [2]Step = read()
print(count(o == Win for o in packed), count(o == Loss for o in spaced))
print(count(grid[i, i] == Loss for i in Game))
for s in steps:
    match s:
        Up(by) => print("up", by)
        Down(by) => print("down", by)
