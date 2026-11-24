// Tuned: the search stops at the first state with the marble on the goal, as the accepted
// C++'s does, where bfs(until:) takes a single target and the port searches every state.
//
// Source: DMOPC '19 Contest 4 P2 (Pleasant Present), https://dmoj.ca/problem/dmopc19c4p2
// Problem: an n x n grid, n <= 15, where all but 2 cells hold magnets; a marble starts on one
//   of the empty cells. Each second either the marble moves into the adjacent empty cell or a
//   magnet slides into it. Print the fewest seconds to get the marble to the goal cell.
// Solution: the state is (marble cell, hole cell), the 15 puzzle with one tile that matters.
//   A move slides the hole one step, swapping with the marble when it is the neighbour. BFS
//   over the n^4 states; the answer is the nearest state with the marble on the goal.
// original: corpus/submissions/5323330.cpp
// Bounds: n: 2..=15; marble.0, marble.1, goal.0, goal.1, empty.0, empty.1: 1..=n
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
// breadth-first by hand, stopping at the first state with the marble on the goal
var dist: [State]Int = -1
var queue = List<State>()
let start = State((marble, hole))
dist[start] = 0
queue.push(start)
var at = 0
while at < queue.len:
    let s = queue[at]
    at += 1
    if s.marble == goal:
        print(dist[s])
        break
    for t in moves(s):
        if dist[t] < 0:
            dist[t] = dist[s] + 1
            queue.push(t)
