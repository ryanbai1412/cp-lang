// DMOPC '19 Contest 4 P2: a token and a hole share an n x n board. One move
// slides the hole one step, or swaps the token into the hole when they are
// adjacent. Fewest moves until the token stands on the target.
// original: corpus/submissions/5323330.cpp
let n: 2..=16 = read()
domain Row = 1..=n
domain Col = 1..=n
type Cell = (Row, Col)
let (s, t, e1, e2): (Cell, Cell, Cell, Cell) = read()

// The token starts on one of the two given cells; the hole is the other.
type State = (token: Cell, hole: Cell)
fn moves(st: State) -> Iter<State>:
    for h in st.hole.adj4:
        yield if h == st.token: (st.hole, st.token) else: (st.token, h)

let dist = bfs(moves, from: [(s, if s == e1: e2 else: e1)])
print(min(dist[t, h] for h in Cell))          // min skips none
