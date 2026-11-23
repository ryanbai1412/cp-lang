// CCO 2019 P1, Human Error: Justin (J) and Donald (D) take turns on a
// small board full of their pieces. A move slides one of your pieces onto
// a neighbouring piece and captures it; whoever cannot move loses. Each
// player picks at random among his best few moves. Print the chance that
// Justin, who moves first, wins.
// original: corpus/submissions/3613457.cpp
//
// At most 13 cells, so search every reachable board. A board's chance for
// the player to move is the mean of `1 - chance` after his `slack` best
// moves (0 with no move); memoize on the board and whose turn it is.
let n: 1..=13 = read()
let m: 1..=13 = read()                                    // at most 13 cells in all
domain Row = 0..n
domain Col = 0..m
type Cell = (Row, Col)

enum Player = J | D
fn other(p: Player) -> Player = if p == J: D else: J
type Board = [Row, Col]?Player                            // None once a piece has left

let letters: [Row, Col]Char = read()
let start: Board = [Row, Col] (r, c) => Some(if letters[r, c] == 'J': J else: D)
let (j_slack, d_slack): (Int, Int) = read()              // how many of his best moves each picks from
fn slack(p: Player) -> Int = if p == J: j_slack else: d_slack

fn key(board: Board, who: Player) -> Int:
    var k = if who == J: 0 else: 1
    for r in Row:
        for c in Col:
            k = k * 3 + (if let p = board[r, c]: (if p == J: 1 else: 2) else: 0)
    k

var memo: Map<Int, F64> = Map()

// The chance that `who`, about to move, wins from here.
fn wins(board: Board, who: Player) -> F64:
    let k = key(board, who)
    if let known = memo.get(k): return known
    var chances: List<F64> = []
    for r in Row:
        for c in Col:
            if board[r, c] != Some(who): continue
            let p: Cell = (r, c)
            for q in p.adj4:
                if board[q] == None: continue
                var next = board
                next[q] = Some(who)
                next[p] = None
                chances.push(1.0 - wins(next, other(who)))
    let best = chances.sorted(desc: true)
    let take = min(best.len, slack(who))
    let chance = if take == 0: 0.0 else: best[0..take].sum() / F64(take)
    memo[k] = chance
    chance

print("{wins(start, J):.3}")
