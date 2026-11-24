// Source: DMOPC '19 Contest 4 P2 (Pleasant Present), https://dmoj.ca/problem/dmopc19c4p2
// Problem: an n x n grid, n <= 15, where all but 2 cells hold magnets; a marble starts on one
//   of the empty cells. Each second either the marble moves into the adjacent empty cell or a
//   magnet slides into it. Print the fewest seconds to get the marble to the goal cell.
// Solution: the state is (marble cell, hole cell), the 15 puzzle with one tile that matters.
//   A move slides the hole one step, swapping with the marble when it is the neighbour. BFS
//   over the n^4 states; the answer is the nearest state with the marble on the goal.
// original: corpus/submissions/5323330.cpp
let n: 2..=15 = read()
domain Row = 1..=n
domain Col = 1..=n
domain Cell = (Row, Col)
domain State = (marble: Cell, hole: Cell)
let marble: Cell = read()
let goal: Cell = read()
let empty: [2]Cell = read()
let hole = if marble == empty[0]: empty[1] else: empty[0]
fn moves(s: State) -> yields State:
    for h in s.hole.adj4:
        yield if h == s.marble: State((s.hole, s.marble)) else: State((s.marble, h))
let dist = bfs(moves, from: [State((marble, hole))])
print(min(dist[State((goal, h))] for h in Cell)!)
